/*******************************************************************************
 *   (c) 2018 - 2023 Zondax AG
 *
 *  Licensed under the Apache License, Version 2.0 (the "License");
 *  you may not use this file except in compliance with the License.
 *  You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 *  Unless required by applicable law or agreed to in writing, software
 *  distributed under the License is distributed on an "AS IS" BASIS,
 *  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *  See the License for the specific language governing permissions and
 *  limitations under the License.
 ********************************************************************************/

#include "parser.h"

#include <stdio.h>
#include <zxformat.h>
#include <zxmacros.h>
#include <zxtypes.h>

#include "action_dutch_auction_end.h"
#include "action_dutch_auction_schedule.h"
#include "coin.h"
#include "crypto.h"
#include "delegate.h"
#include "delegator_vote.h"
#include "ics20_withdrawal.h"
#include "memo.h"
#include "output.h"
#include "parameters.h"
#include "parser_common.h"
#include "parser_impl.h"
#include "position_close.h"
#include "position_open.h"
#include "position_withdraw.h"
#include "spend.h"
#include "swap.h"
#include "tx_metadata.h"
#include "undelegate.h"
#include "undelegate_claim.h"
static uint8_t action_idx = 0;

parser_error_t parser_init_context(parser_context_t *ctx, const uint8_t *buffer, uint16_t bufferSize) {
    ctx->offset = 0;
    ctx->buffer = NULL;
    ctx->bufferLen = 0;

    if (bufferSize == 0 || buffer == NULL) {
        // Not available, use defaults
        return parser_init_context_empty;
    }

    ctx->buffer = buffer;
    ctx->bufferLen = bufferSize;
    return parser_ok;
}

parser_error_t parser_parse(parser_context_t *ctx, const uint8_t *data, size_t dataLen, parser_tx_t *tx_obj) {
    CHECK_ERROR(parser_init_context(ctx, data, dataLen))
    ctx->tx_obj = tx_obj;
    return _read(ctx, tx_obj);
}

parser_error_t parser_validate(parser_context_t *ctx) {
    // Iterate through all items to check that all can be shown and are valid
    uint8_t numItems = 0;
    CHECK_ERROR(parser_getNumItems(ctx, &numItems))

    char tmpKey[40] = {0};
    char tmpVal[40] = {0};

    for (uint8_t idx = 0; idx < numItems; idx++) {
        uint8_t pageCount = 0;
        CHECK_ERROR(parser_getItem(ctx, idx, tmpKey, sizeof(tmpKey), tmpVal, sizeof(tmpVal), 0, &pageCount))
    }
    return parser_ok;
}

parser_error_t parser_getNumItems(const parser_context_t *ctx, uint8_t *num_items) {
    UNUSED(ctx);

    uint8_t parameters_num_items = 0;
    CHECK_ERROR(parameters_getNumItems(ctx, &parameters_num_items))
    *num_items = parameters_num_items;

    uint8_t memo_num_items = 0;
    CHECK_ERROR(memo_getNumItems(ctx, &memo_num_items))
    *num_items += memo_num_items;

    // Add actions number of items
    for (uint8_t i = 0; i < ctx->tx_obj->plan.actions.qty; i++) {
        uint8_t action_num_items = 0;
        switch (ctx->tx_obj->actions_plan[i].action_type) {
            case penumbra_core_transaction_v1_ActionPlan_spend_tag:
                CHECK_ERROR(spend_getNumItems(ctx, &action_num_items));
                break;
            case penumbra_core_transaction_v1_ActionPlan_output_tag:
                CHECK_ERROR(output_getNumItems(ctx, &action_num_items));
                break;
            case penumbra_core_transaction_v1_ActionPlan_ics20_withdrawal_tag:
                CHECK_ERROR(ics20_withdrawal_getNumItems(ctx, &action_num_items));
                break;
#if defined(FULL_APP)
            case penumbra_core_transaction_v1_ActionPlan_swap_tag:
                CHECK_ERROR(swap_getNumItems(ctx, &action_num_items));
                break;
#endif
            case penumbra_core_transaction_v1_ActionPlan_delegate_tag:
                CHECK_ERROR(delegate_getNumItems(ctx, &action_num_items));
                break;
            case penumbra_core_transaction_v1_ActionPlan_undelegate_tag:
                CHECK_ERROR(undelegate_getNumItems(ctx, &action_num_items));
                break;
            case penumbra_core_transaction_v1_ActionPlan_undelegate_claim_tag:
                CHECK_ERROR(undelegate_claim_getNumItems(ctx, &action_num_items));
                break;
            case penumbra_core_transaction_v1_ActionPlan_delegator_vote_tag:
                CHECK_ERROR(delegator_vote_getNumItems(ctx, &action_num_items));
                break;
            case penumbra_core_transaction_v1_ActionPlan_position_open_tag:
                CHECK_ERROR(position_open_getNumItems(ctx, &action_num_items));
                break;
            case penumbra_core_transaction_v1_ActionPlan_position_close_tag:
                CHECK_ERROR(position_close_getNumItems(ctx, &action_num_items));
                break;
            case penumbra_core_transaction_v1_ActionPlan_position_withdraw_tag:
                CHECK_ERROR(position_withdraw_getNumItems(ctx, &action_num_items));
                break;
            case penumbra_core_transaction_v1_ActionPlan_action_dutch_auction_schedule_tag:
                CHECK_ERROR(action_dutch_auction_schedule_getNumItems(ctx, &action_num_items));
                break;
            case penumbra_core_transaction_v1_ActionPlan_action_dutch_auction_end_tag:
                CHECK_ERROR(action_dutch_auction_end_getNumItems(ctx, &action_num_items));
                break;
            default:
                return parser_unexpected_error;
        }
        *num_items += action_num_items;
    }

    if (*num_items == 0) {
        return parser_unexpected_number_items;
    }
    return parser_ok;
}

static void cleanOutput(char *outKey, uint16_t outKeyLen, char *outVal, uint16_t outValLen) {
    MEMZERO(outKey, outKeyLen);
    MEMZERO(outVal, outValLen);
    snprintf(outKey, outKeyLen, "?");
    snprintf(outVal, outValLen, " ");
    action_idx = 0;
}

static parser_error_t checkSanity(uint8_t numItems, uint8_t displayIdx) {
    if (displayIdx >= numItems) {
        return parser_display_idx_out_of_range;
    }
    return parser_ok;
}

parser_error_t parser_getItem(const parser_context_t *ctx, uint8_t displayIdx, char *outKey, uint16_t outKeyLen,
                              char *outVal, uint16_t outValLen, uint8_t pageIdx, uint8_t *pageCount) {
    UNUSED(pageIdx);
    *pageCount = 1;
    uint8_t numItems = 0;
    CHECK_ERROR(parser_getNumItems(ctx, &numItems))
    CHECK_APP_CANARY()

    CHECK_ERROR(checkSanity(numItems, displayIdx))
    cleanOutput(outKey, outKeyLen, outVal, outValLen);

    // Print parameters
    uint8_t parameters_num_items = 0;
    CHECK_ERROR(parameters_getNumItems(ctx, &parameters_num_items))
    if (displayIdx < parameters_num_items) {
        CHECK_ERROR(parameters_getItem(ctx, displayIdx, outKey, outKeyLen, outVal, outValLen, pageIdx, pageCount))
    }

    // Print memo
    uint8_t memo_num_items = 0;
    CHECK_ERROR(memo_getNumItems(ctx, &memo_num_items))
    if (displayIdx >= parameters_num_items && displayIdx < (parameters_num_items + memo_num_items)) {
        CHECK_ERROR(
            memo_getItem(ctx, displayIdx - parameters_num_items, outKey, outKeyLen, outVal, outValLen, pageIdx, pageCount))
    }

    // Print actions
    if (displayIdx >= parameters_num_items + memo_num_items) {
        // Increment action_idx only if displayIdx change
        if (displayIdx != action_idx) {
            action_idx = displayIdx - parameters_num_items - memo_num_items;
        }
        if (action_idx >= ctx->tx_obj->plan.actions.qty) {
            return parser_unexpected_error;
        }
        switch (ctx->tx_obj->actions_plan[action_idx].action_type) {
            case penumbra_core_transaction_v1_ActionPlan_spend_tag:
                CHECK_ERROR(spend_getItem(ctx, &ctx->tx_obj->actions_plan[action_idx].action.spend, action_idx + 1, outKey,
                                          outKeyLen, outVal, outValLen, pageIdx, pageCount))
                break;
            case penumbra_core_transaction_v1_ActionPlan_output_tag:
                CHECK_ERROR(output_getItem(ctx, &ctx->tx_obj->actions_plan[action_idx].action.output, action_idx + 1, outKey,
                                           outKeyLen, outVal, outValLen, pageIdx, pageCount))
                break;
            case penumbra_core_transaction_v1_ActionPlan_ics20_withdrawal_tag:
                CHECK_ERROR(ics20_withdrawal_getItem(ctx, &ctx->tx_obj->actions_plan[action_idx].action.ics20_withdrawal,
                                                     action_idx + 1, outKey, outKeyLen, outVal, outValLen, pageIdx,
                                                     pageCount))
                break;
#if defined(FULL_APP)
            case penumbra_core_transaction_v1_ActionPlan_swap_tag:
                CHECK_ERROR(swap_getItem(ctx, &ctx->tx_obj->actions_plan[action_idx].action.swap, action_idx + 1, outKey,
                                         outKeyLen, outVal, outValLen, pageIdx, pageCount))
                break;
#endif
            case penumbra_core_transaction_v1_ActionPlan_delegate_tag:
                CHECK_ERROR(delegate_getItem(ctx, &ctx->tx_obj->actions_plan[action_idx].action.delegate, action_idx + 1,
                                             outKey, outKeyLen, outVal, outValLen, pageIdx, pageCount))
                break;
            case penumbra_core_transaction_v1_ActionPlan_undelegate_tag:
                CHECK_ERROR(undelegate_getItem(ctx, &ctx->tx_obj->actions_plan[action_idx].action.undelegate, action_idx + 1,
                                               outKey, outKeyLen, outVal, outValLen, pageIdx, pageCount))
                break;
            case penumbra_core_transaction_v1_ActionPlan_undelegate_claim_tag:
                CHECK_ERROR(undelegate_claim_getItem(ctx, &ctx->tx_obj->actions_plan[action_idx].action.undelegate_claim,
                                                     action_idx + 1, outKey, outKeyLen, outVal, outValLen, pageIdx,
                                                     pageCount))
                break;
            case penumbra_core_transaction_v1_ActionPlan_delegator_vote_tag:
                CHECK_ERROR(delegator_vote_getItem(ctx, &ctx->tx_obj->actions_plan[action_idx].action.delegator_vote,
                                                   action_idx + 1, outKey, outKeyLen, outVal, outValLen, pageIdx, pageCount))
                break;
            case penumbra_core_transaction_v1_ActionPlan_position_open_tag:
                CHECK_ERROR(position_open_getItem(ctx, &ctx->tx_obj->actions_plan[action_idx].action.position_open,
                                                  action_idx + 1, outKey, outKeyLen, outVal, outValLen, pageIdx, pageCount))
                break;
            case penumbra_core_transaction_v1_ActionPlan_position_close_tag:
                CHECK_ERROR(position_close_getItem(ctx, &ctx->tx_obj->actions_plan[action_idx].action.position_close,
                                                   action_idx + 1, outKey, outKeyLen, outVal, outValLen, pageIdx, pageCount))
                break;
            case penumbra_core_transaction_v1_ActionPlan_position_withdraw_tag:
                CHECK_ERROR(position_withdraw_getItem(ctx, &ctx->tx_obj->actions_plan[action_idx].action.position_withdraw,
                                                      action_idx + 1, outKey, outKeyLen, outVal, outValLen, pageIdx,
                                                      pageCount))
                break;
            case penumbra_core_transaction_v1_ActionPlan_action_dutch_auction_schedule_tag:
                CHECK_ERROR(action_dutch_auction_schedule_getItem(
                    ctx, &ctx->tx_obj->actions_plan[action_idx].action.action_dutch_auction_schedule, action_idx + 1, outKey,
                    outKeyLen, outVal, outValLen, pageIdx, pageCount))
                break;
            case penumbra_core_transaction_v1_ActionPlan_action_dutch_auction_end_tag:
                CHECK_ERROR(action_dutch_auction_end_getItem(
                    ctx, &ctx->tx_obj->actions_plan[action_idx].action.action_dutch_auction_end, action_idx + 1, outKey,
                    outKeyLen, outVal, outValLen, pageIdx, pageCount))
                break;
            default:
                return parser_unexpected_error;
        }
    }

    return parser_ok;
}

parser_error_t parser_parseTxMetadata(const uint8_t *data, size_t dataLen, tx_metadata_t *metadata, uint8_t metadataLen) {
    return metadata_parse(data, dataLen, metadata, metadataLen);
}
