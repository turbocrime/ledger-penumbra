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
#include "action_dutch_auction_schedule.h"

#include "note.h"
#include "parser_pb_utils.h"
#include "ui_utils.h"
#include "zxformat.h"

parser_error_t decode_action_dutch_auction_schedule_plan(
    const bytes_t *data, action_dutch_auction_schedule_plan_t *action_dutch_auction_schedule) {
    penumbra_core_component_auction_v1_ActionDutchAuctionSchedule action_dutch_auction_schedule_pb =
        penumbra_core_component_auction_v1_ActionDutchAuctionSchedule_init_default;

    pb_istream_t stream = pb_istream_from_buffer(data->ptr, data->len);
    CHECK_APP_CANARY()

    // Set up fixed size fields
    fixed_size_field_t nonce_arg, input_asset_id_arg, output_asset_id_arg;

    setup_decode_fixed_field(&action_dutch_auction_schedule_pb.description.nonce, &nonce_arg,
                             &action_dutch_auction_schedule->description.nonce, 32);
    setup_decode_fixed_field(&action_dutch_auction_schedule_pb.description.input.asset_id.inner, &input_asset_id_arg,
                             &action_dutch_auction_schedule->description.input.asset_id.inner, ASSET_ID_LEN);
    setup_decode_fixed_field(&action_dutch_auction_schedule_pb.description.output_id.inner, &output_asset_id_arg,
                             &action_dutch_auction_schedule->description.output_id.inner, ASSET_ID_LEN);

    if (!pb_decode(&stream, penumbra_core_component_auction_v1_ActionDutchAuctionSchedule_fields,
                   &action_dutch_auction_schedule_pb)) {
        return parser_action_dutch_auction_schedule_plan_error;
    }

    action_dutch_auction_schedule->has_description = action_dutch_auction_schedule_pb.has_description;
    if (action_dutch_auction_schedule_pb.has_description) {
        action_dutch_auction_schedule->description.has_input = action_dutch_auction_schedule_pb.description.has_input;
        if (action_dutch_auction_schedule->description.has_input) {
            action_dutch_auction_schedule->description.input.has_amount =
                action_dutch_auction_schedule_pb.description.input.has_amount;
            if (action_dutch_auction_schedule_pb.description.input.has_amount) {
                action_dutch_auction_schedule->description.input.amount.hi =
                    action_dutch_auction_schedule_pb.description.input.amount.hi;
                action_dutch_auction_schedule->description.input.amount.lo =
                    action_dutch_auction_schedule_pb.description.input.amount.lo;
            }
            action_dutch_auction_schedule->description.input.has_asset_id =
                action_dutch_auction_schedule_pb.description.input.has_asset_id;
        }
        action_dutch_auction_schedule->description.has_output_id =
            action_dutch_auction_schedule_pb.description.has_output_id;
        action_dutch_auction_schedule->description.has_max_output =
            action_dutch_auction_schedule_pb.description.has_max_output;
        if (action_dutch_auction_schedule_pb.description.has_max_output) {
            action_dutch_auction_schedule->description.max_output.hi =
                action_dutch_auction_schedule_pb.description.max_output.hi;
            action_dutch_auction_schedule->description.max_output.lo =
                action_dutch_auction_schedule_pb.description.max_output.lo;
        }
        action_dutch_auction_schedule->description.has_min_output =
            action_dutch_auction_schedule_pb.description.has_min_output;
        if (action_dutch_auction_schedule_pb.description.has_min_output) {
            action_dutch_auction_schedule->description.min_output.hi =
                action_dutch_auction_schedule_pb.description.min_output.hi;
            action_dutch_auction_schedule->description.min_output.lo =
                action_dutch_auction_schedule_pb.description.min_output.lo;
        }
        action_dutch_auction_schedule->description.start_height = action_dutch_auction_schedule_pb.description.start_height;
        action_dutch_auction_schedule->description.end_height = action_dutch_auction_schedule_pb.description.end_height;
        action_dutch_auction_schedule->description.step_count = action_dutch_auction_schedule_pb.description.step_count;
    }

    return parser_ok;
}

parser_error_t action_dutch_auction_schedule_getNumItems(const parser_context_t *ctx, uint8_t *num_items) {
    UNUSED(ctx);
    *num_items = 1;
    return parser_ok;
}

parser_error_t action_dutch_auction_schedule_getItem(
    const parser_context_t *ctx, const action_dutch_auction_schedule_plan_t *action_dutch_auction_schedule,
    uint8_t actionIdx, char *outKey, uint16_t outKeyLen, char *outVal, uint16_t outValLen, uint8_t pageIdx,
    uint8_t *pageCount) {
    parser_error_t err = parser_no_data;
    if (action_dutch_auction_schedule == NULL || outKey == NULL || outVal == NULL || outKeyLen == 0 || outValLen == 0) {
        return err;
    }

    char bufferUI[DUTCH_AUCTION_SCHEDULE_DISPLAY_MAX_LEN] = {0};

    snprintf(outKey, outKeyLen, "Action_%d", actionIdx);
    CHECK_ERROR(action_dutch_auction_schedule_printValue(ctx, action_dutch_auction_schedule, bufferUI, sizeof(bufferUI)));
    pageString(outVal, outValLen, bufferUI, pageIdx, pageCount);

    return parser_ok;
}

parser_error_t action_dutch_auction_schedule_printValue(
    const parser_context_t *ctx, const action_dutch_auction_schedule_plan_t *action_dutch_auction_schedule, char *outVal,
    uint16_t outValLen) {
    if (ctx == NULL || action_dutch_auction_schedule == NULL || outVal == NULL) {
        return parser_no_data;
    }

    if (outValLen < DUTCH_AUCTION_SCHEDULE_DISPLAY_MAX_LEN) {
        return parser_unexpected_buffer_end;
    }

    MEMZERO(outVal, outValLen);

    // add action title
    snprintf(outVal, outValLen, "DutchAuctionSchedule Selling: ");
    uint16_t written_value = strlen(outVal);

    // Selling value
    CHECK_ERROR(printValue(ctx, &action_dutch_auction_schedule->description.input, &ctx->tx_obj->parameters_plan.chain_id,
                           true, outVal + written_value, outValLen - written_value));
    written_value = strlen(outVal);

    snprintf(outVal + written_value, outValLen - written_value, " For: ");
    written_value = strlen(outVal);

    // for asset
    value_t for_asset = {.amount = {0},
                         .asset_id.inner = {.ptr = action_dutch_auction_schedule->description.output_id.inner.ptr,
                                            .len = action_dutch_auction_schedule->description.output_id.inner.len},
                         .has_amount = false,
                         .has_asset_id = true};
    CHECK_ERROR(printValue(ctx, &for_asset, &ctx->tx_obj->parameters_plan.chain_id, true, outVal + written_value,
                           outValLen - written_value));
    written_value = strlen(outVal);

    // Format starting price
    snprintf(outVal + written_value, outValLen - written_value, " Starting price: ");
    written_value = strlen(outVal);

    value_t start_price_1 = {.amount = action_dutch_auction_schedule->description.max_output,
                             .asset_id.inner = {.ptr = action_dutch_auction_schedule->description.output_id.inner.ptr,
                                                .len = action_dutch_auction_schedule->description.output_id.inner.len},
                             .has_amount = true,
                             .has_asset_id = true};
    CHECK_ERROR(printValue(ctx, &start_price_1, &ctx->tx_obj->parameters_plan.chain_id, false, outVal + written_value,
                           outValLen - written_value));
    written_value = strlen(outVal);

    snprintf(outVal + written_value, outValLen - written_value, " for ");
    written_value = strlen(outVal);

    CHECK_ERROR(printValue(ctx, &action_dutch_auction_schedule->description.input, &ctx->tx_obj->parameters_plan.chain_id,
                           false, outVal + written_value, outValLen - written_value));
    written_value = strlen(outVal);

    // Format ending price
    snprintf(outVal + written_value, outValLen - written_value, " Ending price: ");
    written_value = strlen(outVal);

    value_t end_price_1 = {.amount = action_dutch_auction_schedule->description.min_output,
                           .asset_id.inner = {.ptr = action_dutch_auction_schedule->description.output_id.inner.ptr,
                                              .len = action_dutch_auction_schedule->description.output_id.inner.len},
                           .has_amount = true,
                           .has_asset_id = true};
    CHECK_ERROR(printValue(ctx, &end_price_1, &ctx->tx_obj->parameters_plan.chain_id, false, outVal + written_value,
                           outValLen - written_value));
    written_value = strlen(outVal);

    snprintf(outVal + written_value, outValLen - written_value, " for ");
    written_value = strlen(outVal);

    CHECK_ERROR(printValue(ctx, &action_dutch_auction_schedule->description.input, &ctx->tx_obj->parameters_plan.chain_id,
                           false, outVal + written_value, outValLen - written_value));
    written_value = strlen(outVal);

    // Start block height
    snprintf(outVal + written_value, outValLen - written_value, " Start block height: ");
    written_value = strlen(outVal);
    snprintf(outVal + written_value, outValLen - written_value, "%llu",
             action_dutch_auction_schedule->description.start_height);
    written_value = strlen(outVal);

    // End block height
    snprintf(outVal + written_value, outValLen - written_value, " End block height: ");
    written_value = strlen(outVal);
    snprintf(outVal + written_value, outValLen - written_value, "%llu",
             action_dutch_auction_schedule->description.end_height);
    written_value = strlen(outVal);

    // Step count
    snprintf(outVal + written_value, outValLen - written_value, " Steps: ");
    written_value = strlen(outVal);
    snprintf(outVal + written_value, outValLen - written_value, "%llu",
             action_dutch_auction_schedule->description.step_count);

    return parser_ok;
}
