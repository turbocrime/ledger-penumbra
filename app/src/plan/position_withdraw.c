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
#include "position_withdraw.h"

#include "note.h"
#include "parser_pb_utils.h"
#include "ui_utils.h"
#include "zxformat.h"

parser_error_t decode_position_withdraw_plan(const bytes_t *data, position_withdraw_plan_t *position_withdraw) {
    penumbra_core_component_dex_v1_PositionWithdrawPlan position_withdraw_pb =
        penumbra_core_component_dex_v1_PositionWithdrawPlan_init_default;

    pb_istream_t stream = pb_istream_from_buffer(data->ptr, data->len);
    CHECK_APP_CANARY()

    // Set up fixed size fields
    fixed_size_field_t position_id_arg, pair_asset_1_arg, pair_asset_2_arg;
    setup_decode_fixed_field(&position_withdraw_pb.position_id.inner, &position_id_arg,
                             &position_withdraw->position_id.inner, POSITION_ID_LEN);
    setup_decode_fixed_field(&position_withdraw_pb.pair.asset_1.inner, &pair_asset_1_arg,
                             &position_withdraw->pair.asset_1.inner, ASSET_ID_LEN);
    setup_decode_fixed_field(&position_withdraw_pb.pair.asset_2.inner, &pair_asset_2_arg,
                             &position_withdraw->pair.asset_2.inner, ASSET_ID_LEN);

    variable_size_field_array_t rewards_arg;
    bytes_t rewards_bytes[MAX_CALLBACK_ARRAY_SIZE];
    setup_decode_variable_field_array(&position_withdraw_pb.rewards, &rewards_arg, rewards_bytes, MAX_CALLBACK_ARRAY_SIZE);

    if (!pb_decode(&stream, penumbra_core_component_dex_v1_PositionWithdrawPlan_fields, &position_withdraw_pb)) {
        return parser_position_withdraw_plan_error;
    }
    CHECK_APP_CANARY()
    position_withdraw->rewards_qty = rewards_arg.filled_count;

    // decode rewards
    for (uint8_t i = 0; i < position_withdraw->rewards_qty; i++) {
        penumbra_core_asset_v1_Value value_pb = penumbra_core_asset_v1_Value_init_default;
        pb_istream_t stream_value = pb_istream_from_buffer(rewards_bytes[i].ptr, rewards_bytes[i].len);
        fixed_size_field_t asset_id_arg;
        setup_decode_fixed_field(&value_pb.asset_id.inner, &asset_id_arg, &position_withdraw->rewards[i].asset_id.inner,
                                 ASSET_ID_LEN);
        if (!pb_decode(&stream_value, penumbra_core_asset_v1_Value_fields, &value_pb)) {
            return parser_position_withdraw_plan_error;
        }
        position_withdraw->rewards[i].has_asset_id = value_pb.has_asset_id;
        position_withdraw->rewards[i].has_amount = value_pb.has_amount;
        if (position_withdraw->rewards[i].has_amount) {
            position_withdraw->rewards[i].amount.hi = value_pb.amount.hi;
            position_withdraw->rewards[i].amount.lo = value_pb.amount.lo;
        }
    }
    CHECK_APP_CANARY()

    position_withdraw->has_reserves = position_withdraw_pb.has_reserves;
    if (position_withdraw->has_reserves) {
        position_withdraw->reserves.has_r1 = position_withdraw_pb.reserves.has_r1;
        if (position_withdraw->reserves.has_r1) {
            position_withdraw->reserves.r1.hi = position_withdraw_pb.reserves.r1.hi;
            position_withdraw->reserves.r1.lo = position_withdraw_pb.reserves.r1.lo;
        }
        position_withdraw->reserves.has_r2 = position_withdraw_pb.reserves.has_r2;
        if (position_withdraw->reserves.has_r2) {
            position_withdraw->reserves.r2.hi = position_withdraw_pb.reserves.r2.hi;
            position_withdraw->reserves.r2.lo = position_withdraw_pb.reserves.r2.lo;
        }
    }
    position_withdraw->has_position_id = position_withdraw_pb.has_position_id;

    position_withdraw->has_pair = position_withdraw_pb.has_pair;
    if (position_withdraw->has_pair) {
        position_withdraw->pair.has_asset_1 = position_withdraw_pb.pair.has_asset_1;
        position_withdraw->pair.has_asset_2 = position_withdraw_pb.pair.has_asset_2;
    }
    position_withdraw->sequence = position_withdraw_pb.sequence;

    return parser_ok;
}

parser_error_t position_withdraw_getNumItems(const parser_context_t *ctx, uint8_t *num_items) {
    UNUSED(ctx);
    *num_items = 1;
    return parser_ok;
}

parser_error_t position_withdraw_getItem(const parser_context_t *ctx, const position_withdraw_plan_t *position_withdraw,
                                         uint8_t actionIdx, char *outKey, uint16_t outKeyLen, char *outVal,
                                         uint16_t outValLen, uint8_t pageIdx, uint8_t *pageCount) {
    parser_error_t err = parser_no_data;
    if (position_withdraw == NULL || outKey == NULL || outVal == NULL || outKeyLen == 0 || outValLen == 0) {
        return err;
    }

    char bufferUI[POSITION_WITHDRAW_DISPLAY_MAX_LEN] = {0};

    snprintf(outKey, outKeyLen, "Action_%d", actionIdx);
    CHECK_ERROR(position_withdraw_printValue(ctx, position_withdraw, bufferUI, sizeof(bufferUI)));
    pageString(outVal, outValLen, bufferUI, pageIdx, pageCount);

    return parser_ok;
}

parser_error_t position_withdraw_printValue(const parser_context_t *ctx, const position_withdraw_plan_t *position_withdraw,
                                            char *outVal, uint16_t outValLen) {
    if (ctx == NULL || position_withdraw == NULL || outVal == NULL) {
        return parser_no_data;
    }

    if (outValLen < POSITION_WITHDRAW_DISPLAY_MAX_LEN) {
        return parser_unexpected_buffer_end;
    }

    MEMZERO(outVal, outValLen);

    // add action title
    snprintf(outVal, outValLen, "PositionWithdraw Position ID ");
    uint16_t written_value = strlen(outVal);

    // add position id
    MEMCPY(outVal + written_value, position_withdraw->position_id.inner.ptr, position_withdraw->position_id.inner.len);
    CHECK_ERROR(encodePositionId(position_withdraw->position_id.inner.ptr, position_withdraw->position_id.inner.len,
                                 outVal + written_value, outValLen - written_value));
    written_value = strlen(outVal);

    // add sequence number
    snprintf(outVal + written_value, outValLen - written_value, " Sequence number ");
    written_value = strlen(outVal);
    snprintf(outVal + written_value, outValLen - written_value, "%llu", position_withdraw->sequence);

    return parser_ok;
}
