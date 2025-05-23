/*******************************************************************************
 *  (c) 2018 - 2023 Zondax AG
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
#include "action_dutch_auction_withdraw.h"

#include "note.h"
#include "parser_pb_utils.h"
#include "ui_utils.h"
#include "zxformat.h"

parser_error_t decode_action_dutch_auction_withdraw_plan(
    const bytes_t *data, action_dutch_auction_withdraw_plan_t *action_dutch_auction_withdraw) {
    penumbra_core_component_auction_v1_ActionDutchAuctionWithdrawPlan action_dutch_auction_withdraw_pb =
        penumbra_core_component_auction_v1_ActionDutchAuctionWithdrawPlan_init_default;

    pb_istream_t stream = pb_istream_from_buffer(data->ptr, data->len);
    CHECK_APP_CANARY()

    // Set up fixed size fields
    fixed_size_field_t position_id_arg, pair_asset_1_arg, pair_asset_2_arg;
    setup_decode_fixed_field(&action_dutch_auction_withdraw_pb.auction_id.inner, &position_id_arg,
                             &action_dutch_auction_withdraw->auction_id.inner, POSITION_ID_LEN);
    setup_decode_fixed_field(&action_dutch_auction_withdraw_pb.reserves_input.asset_id.inner, &pair_asset_1_arg,
                             &action_dutch_auction_withdraw->reserves_input.asset_id.inner, ASSET_ID_LEN);
    setup_decode_fixed_field(&action_dutch_auction_withdraw_pb.reserves_output.asset_id.inner, &pair_asset_2_arg,
                             &action_dutch_auction_withdraw->reserves_output.asset_id.inner, ASSET_ID_LEN);

    if (!pb_decode(&stream, penumbra_core_component_auction_v1_ActionDutchAuctionWithdrawPlan_fields,
                   &action_dutch_auction_withdraw_pb)) {
        return parser_action_dutch_auction_withdraw_plan_error;
    }

    action_dutch_auction_withdraw->has_auction_id = action_dutch_auction_withdraw_pb.has_auction_id;
    action_dutch_auction_withdraw->seq = action_dutch_auction_withdraw_pb.seq;
    action_dutch_auction_withdraw->has_reserves_input = action_dutch_auction_withdraw_pb.has_reserves_input;
    if (action_dutch_auction_withdraw->has_reserves_input) {
        action_dutch_auction_withdraw->reserves_input.has_amount =
            action_dutch_auction_withdraw_pb.reserves_input.has_amount;
        if (action_dutch_auction_withdraw->reserves_input.has_amount) {
            action_dutch_auction_withdraw->reserves_input.amount.hi =
                action_dutch_auction_withdraw_pb.reserves_input.amount.hi;
            action_dutch_auction_withdraw->reserves_input.amount.lo =
                action_dutch_auction_withdraw_pb.reserves_input.amount.lo;
        }
        action_dutch_auction_withdraw->reserves_input.has_asset_id =
            action_dutch_auction_withdraw_pb.reserves_input.has_asset_id;
    }
    action_dutch_auction_withdraw->has_reserves_output = action_dutch_auction_withdraw_pb.has_reserves_output;
    if (action_dutch_auction_withdraw->has_reserves_output) {
        action_dutch_auction_withdraw->reserves_output.has_amount =
            action_dutch_auction_withdraw_pb.reserves_output.has_amount;
        if (action_dutch_auction_withdraw->reserves_output.has_amount) {
            action_dutch_auction_withdraw->reserves_output.amount.hi =
                action_dutch_auction_withdraw_pb.reserves_output.amount.hi;
            action_dutch_auction_withdraw->reserves_output.amount.lo =
                action_dutch_auction_withdraw_pb.reserves_output.amount.lo;
        }
        action_dutch_auction_withdraw->reserves_output.has_asset_id =
            action_dutch_auction_withdraw_pb.reserves_output.has_asset_id;
    }

    return parser_ok;
}

parser_error_t action_dutch_auction_withdraw_getNumItems(const parser_context_t *ctx, uint8_t *num_items) {
    UNUSED(ctx);
    *num_items = 1;
    return parser_ok;
}

parser_error_t action_dutch_auction_withdraw_getItem(
    const parser_context_t *ctx, const action_dutch_auction_withdraw_plan_t *action_dutch_auction_withdraw,
    uint8_t actionIdx, char *outKey, uint16_t outKeyLen, char *outVal, uint16_t outValLen, uint8_t pageIdx,
    uint8_t *pageCount) {
    parser_error_t err = parser_no_data;
    if (action_dutch_auction_withdraw == NULL || outKey == NULL || outVal == NULL || outKeyLen == 0 || outValLen == 0) {
        return err;
    }

    char bufferUI[DUTCH_AUCTION_WITHDRAW_DISPLAY_MAX_LEN] = {0};

    snprintf(outKey, outKeyLen, "Action_%d", actionIdx + 1);
    CHECK_ERROR(
        action_dutch_auction_withdraw_printValue(ctx, action_dutch_auction_withdraw, bufferUI, sizeof(bufferUI)));
    pageString(outVal, outValLen, bufferUI, pageIdx, pageCount);

    return parser_ok;
}

parser_error_t action_dutch_auction_withdraw_printValue(
    const parser_context_t *ctx, const action_dutch_auction_withdraw_plan_t *action_dutch_auction_withdraw,
    char *outVal, uint16_t outValLen) {
    if (ctx == NULL || action_dutch_auction_withdraw == NULL || outVal == NULL) {
        return parser_no_data;
    }

    if (outValLen < DUTCH_AUCTION_WITHDRAW_DISPLAY_MAX_LEN) {
        return parser_unexpected_buffer_end;
    }

    MEMZERO(outVal, outValLen);

    // add action title
    snprintf(outVal, outValLen, "DutchAuctionWithdraw Auction ID: ");
    uint16_t written_value = strlen(outVal);

    // add auction id
    MEMCPY(outVal + written_value, action_dutch_auction_withdraw->auction_id.inner.ptr,
           action_dutch_auction_withdraw->auction_id.inner.len);

    CHECK_ERROR(encodeAuctionId(action_dutch_auction_withdraw->auction_id.inner.ptr,
                                action_dutch_auction_withdraw->auction_id.inner.len, outVal + written_value,
                                outValLen - written_value));
    written_value = strlen(outVal);

    // add unsold amount
    snprintf(outVal + written_value, outValLen - written_value, " Unsold: ");
    written_value = strlen(outVal);
    CHECK_ERROR(printValue(ctx, &action_dutch_auction_withdraw->reserves_input, &ctx->tx_obj->parameters_plan.chain_id,
                           true, outVal + written_value, outValLen - written_value));
    written_value = strlen(outVal);

    // add proceeds amount
    snprintf(outVal + written_value, outValLen - written_value, " Proceeds: ");
    written_value = strlen(outVal);
    CHECK_ERROR(printValue(ctx, &action_dutch_auction_withdraw->reserves_output, &ctx->tx_obj->parameters_plan.chain_id,
                           true, outVal + written_value, outValLen - written_value));
    written_value = strlen(outVal);

    // add sequence number
    snprintf(outVal + written_value, outValLen - written_value, " Sequence number: ");
    written_value = strlen(outVal);
    uint64_to_str(outVal + written_value, outValLen - written_value, action_dutch_auction_withdraw->seq);

    return parser_ok;
}
