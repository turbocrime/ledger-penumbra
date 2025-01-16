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

#include "note.h"
#include "parser_pb_utils.h"
#include "ui_utils.h"
#include "zxformat.h"

parser_error_t decode_spend_plan(const bytes_t *data, spend_plan_t *output) {
    penumbra_core_component_shielded_pool_v1_SpendPlan spend_plan =
        penumbra_core_component_shielded_pool_v1_SpendPlan_init_default;

    pb_istream_t stream = pb_istream_from_buffer(data->ptr, data->len);
    CHECK_APP_CANARY()

    // Set up fixed size fields
    fixed_size_field_t randomizer_arg, value_blinding_arg, proof_blinding_r_arg, proof_blinding_s_arg;

    setup_decode_fixed_field(&spend_plan.randomizer, &randomizer_arg, &output->randomizer, 32);
    setup_decode_fixed_field(&spend_plan.value_blinding, &value_blinding_arg, &output->value_blinding, 32);
    setup_decode_fixed_field(&spend_plan.proof_blinding_r, &proof_blinding_r_arg, &output->proof_blinding_r, 32);
    setup_decode_fixed_field(&spend_plan.proof_blinding_s, &proof_blinding_s_arg, &output->proof_blinding_s, 32);

    // inner in address
    fixed_size_field_t address_inner_arg;
    setup_decode_fixed_field(&spend_plan.note.address.inner, &address_inner_arg, &output->note.address.inner, 80);

    // asset_id in Note
    fixed_size_field_t asset_id_arg;
    setup_decode_fixed_field(&spend_plan.note.value.asset_id.inner, &asset_id_arg, &output->note.value.asset_id.inner,
                             ASSET_ID_LEN);
    // rseed in Note
    fixed_size_field_t rseed_arg;
    setup_decode_fixed_field(&spend_plan.note.rseed, &rseed_arg, &output->note.rseed, RSEED_LEN);

    if (!pb_decode(&stream, penumbra_core_component_shielded_pool_v1_SpendPlan_fields, &spend_plan)) {
        return parser_spend_plan_error;
    }

    output->note.has_value = spend_plan.note.has_value;
    if (output->note.has_value) {
        output->note.value.has_amount = spend_plan.note.value.has_amount;
        if (output->note.value.has_amount) {
            output->note.value.amount.lo = spend_plan.note.value.amount.lo;
            output->note.value.amount.hi = spend_plan.note.value.amount.hi;
        }
    }
    output->note.has_address = spend_plan.note.has_address;
    output->note.value.has_asset_id = spend_plan.note.value.has_asset_id;
    output->position = spend_plan.position;

    return parser_ok;
}

parser_error_t spend_printValue(const parser_context_t *ctx, const spend_plan_t *spend, char *outVal, uint16_t outValLen) {
    if (ctx == NULL || spend == NULL || outVal == NULL) {
        return parser_no_data;
    }

    if (outValLen < SPEND_DISPLAY_MAX_LEN) {
        return parser_unexpected_buffer_end;
    }

    MEMZERO(outVal, outValLen);

    // example: Spend 100 USDC to penumbra1k0zzug62gpz60sejdvu9q7mq…

    // add action title
    snprintf(outVal, outValLen, "%s", "Spend ");
    uint16_t written_value = strlen(outVal);

    // add value
    CHECK_ERROR(printValue(ctx, &spend->note.value, &ctx->tx_obj->parameters_plan.chain_id, outVal + written_value,
                           outValLen - written_value));
    written_value = strlen(outVal);

    // add "from"
    snprintf(outVal + written_value, outValLen - written_value, " from ");
    written_value = strlen(outVal);

    // add address
    CHECK_ERROR(printTxAddress(&spend->note.address.inner, outVal + written_value, outValLen - written_value));

    return parser_ok;
}

parser_error_t spend_getNumItems(const parser_context_t *ctx, uint8_t *num_items) {
    UNUSED(ctx);
    // from spends we display only two items:
    // - Spend 100 USDC
    // - From Main Account
    // all concatenated in a single string
    *num_items = 1;
    return parser_ok;
}

parser_error_t spend_getItem(const parser_context_t *ctx, const spend_plan_t *spend, uint8_t actionIdx, char *outKey,
                             uint16_t outKeyLen, char *outVal, uint16_t outValLen, uint8_t pageIdx, uint8_t *pageCount) {
    parser_error_t err = parser_no_data;

    if (spend == NULL || outKey == NULL || outVal == NULL || outKeyLen == 0 || outValLen == 0) {
        return err;
    }

    char bufferUI[SPEND_DISPLAY_MAX_LEN] = {0};

    snprintf(outKey, outKeyLen, "Action_%d", actionIdx);
    CHECK_ERROR(spend_printValue(ctx, spend, bufferUI, sizeof(bufferUI)));
    pageString(outVal, outValLen, bufferUI, pageIdx, pageCount);

    return parser_ok;
}
