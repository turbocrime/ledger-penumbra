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
#include "ics20_withdrawal.h"

#include "note.h"
#include "parser_pb_utils.h"
#include "rslib.h"
#include "ui_utils.h"
#include "zxformat.h"

parser_error_t decode_ics20_withdrawal_plan(const bytes_t *data, ics20_withdrawal_plan_t *withdrawal) {
    penumbra_core_component_ibc_v1_Ics20Withdrawal withdrawal_plan =
        penumbra_core_component_ibc_v1_Ics20Withdrawal_init_default;

    pb_istream_t withdrawal_stream = pb_istream_from_buffer(data->ptr, data->len);
    CHECK_APP_CANARY()

    // Set up fixed size fields
    fixed_size_field_t return_address_arg;
    setup_decode_fixed_field(&withdrawal_plan.return_address.inner, &return_address_arg, &withdrawal->return_address.inner,
                             80);

    // Set up variable size fields
    variable_size_field_t denom_arg, destination_chain_address_arg, source_channel_arg;
    setup_decode_variable_field(&withdrawal_plan.denom.denom, &denom_arg, &withdrawal->denom.inner);
    setup_decode_variable_field(&withdrawal_plan.destination_chain_address, &destination_chain_address_arg,
                                &withdrawal->destination_chain_address);
    setup_decode_variable_field(&withdrawal_plan.source_channel, &source_channel_arg, &withdrawal->source_channel);

    if (!pb_decode(&withdrawal_stream, penumbra_core_component_ibc_v1_Ics20Withdrawal_fields, &withdrawal_plan)) {
        return parser_ics20_withdrawal_plan_error;
    }

    withdrawal->has_amount = withdrawal_plan.has_amount;
    if (withdrawal_plan.has_amount) {
        withdrawal->amount.lo = withdrawal_plan.amount.lo;
        withdrawal->amount.hi = withdrawal_plan.amount.hi;
    }
    withdrawal->has_denom = withdrawal_plan.has_denom;
    withdrawal->has_return_address = withdrawal_plan.has_return_address;
    withdrawal->has_timeout_height = withdrawal_plan.has_timeout_height;
    if (withdrawal_plan.has_timeout_height) {
        withdrawal->timeout_height.revision_number = withdrawal_plan.timeout_height.revision_number;
        withdrawal->timeout_height.revision_height = withdrawal_plan.timeout_height.revision_height;
    }
    withdrawal->timeout_time = withdrawal_plan.timeout_time;
    withdrawal->use_compat_address = withdrawal_plan.use_compat_address;

    return parser_ok;
}

parser_error_t ics20_withdrawal_getNumItems(const parser_context_t *ctx, uint8_t *num_items) {
    UNUSED(ctx);
    *num_items = 1;
    return parser_ok;
}

parser_error_t ics20_withdrawal_getItem(const parser_context_t *ctx, const ics20_withdrawal_plan_t *ics20_withdrawal,
                                        uint8_t actionIdx, char *outKey, uint16_t outKeyLen, char *outVal,
                                        uint16_t outValLen, uint8_t pageIdx, uint8_t *pageCount) {
    parser_error_t err = parser_no_data;
    if (ics20_withdrawal == NULL || outKey == NULL || outVal == NULL || outKeyLen == 0 || outValLen == 0) {
        return err;
    }

    char bufferUI[ICS20_WITHDRAWAL_DISPLAY_MAX_LEN] = {0};

    snprintf(outKey, outKeyLen, "Action_%d", actionIdx);
    CHECK_ERROR(ics20_withdrawal_printValue(ctx, ics20_withdrawal, bufferUI, sizeof(bufferUI)));
    pageString(outVal, outValLen, bufferUI, pageIdx, pageCount);

    return parser_ok;
}

parser_error_t ics20_withdrawal_printValue(const parser_context_t *ctx, const ics20_withdrawal_plan_t *ics20_withdrawal,
                                           char *outVal, uint16_t outValLen) {
    if (ctx == NULL || ics20_withdrawal == NULL || outVal == NULL) {
        return parser_no_data;
    }

    if (outValLen < ICS20_WITHDRAWAL_DISPLAY_MAX_LEN) {
        return parser_unexpected_buffer_end;
    }

    MEMZERO(outVal, outValLen);

    // add action title
    snprintf(outVal, outValLen, "ICS20Withdrawal ");
    uint16_t written_value = strlen(outVal);

    // add "channel"
    snprintf(outVal + written_value, outValLen - written_value, "Channel ");
    written_value = strlen(outVal);

    MEMCPY(outVal + written_value, ics20_withdrawal->source_channel.ptr, ics20_withdrawal->source_channel.len);
    written_value += ics20_withdrawal->source_channel.len;

    snprintf(outVal + written_value, outValLen - written_value, " Amount ");
    written_value = strlen(outVal);

    uint8_t asset_id_bytes[ASSET_ID_LEN] = {0};
    rs_get_asset_id_from_metadata(&ics20_withdrawal->denom.inner, asset_id_bytes, ASSET_ID_LEN);

    value_t ics20_withdrawal_value = {0};
    ics20_withdrawal_value.amount.hi = ics20_withdrawal->amount.hi;
    ics20_withdrawal_value.amount.lo = ics20_withdrawal->amount.lo;
    ics20_withdrawal_value.asset_id.inner.ptr = asset_id_bytes;
    ics20_withdrawal_value.asset_id.inner.len = ASSET_ID_LEN;
    ics20_withdrawal_value.has_amount = true;
    ics20_withdrawal_value.has_asset_id = true;
    CHECK_ERROR(printValue(ctx, &ics20_withdrawal_value, &ctx->tx_obj->parameters_plan.chain_id, outVal + written_value,
                           outValLen - written_value));
    written_value = strlen(outVal);

    snprintf(outVal + written_value, outValLen - written_value, " To ");
    written_value = strlen(outVal);

    MEMCPY(outVal + written_value, ics20_withdrawal->destination_chain_address.ptr,
           ics20_withdrawal->destination_chain_address.len);

    return parser_ok;
}
