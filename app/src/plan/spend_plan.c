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

#include "parser_impl.h"
#include "parser_interface.h"
#include "parser_pb_utils.h"
#include "pb_common.h"
#include "pb_decode.h"
#include "protobuf/penumbra/core/transaction/v1/transaction.pb.h"
#include "zxformat.h"
#include "known_assets.h"
#include "note.h"

parser_error_t decode_spend_plan(const bytes_t *data, spend_plan_t *output) {
    penumbra_core_component_shielded_pool_v1_SpendPlan spend_plan =
        penumbra_core_component_shielded_pool_v1_SpendPlan_init_default;

    pb_istream_t spend_stream = pb_istream_from_buffer(data->ptr, data->len);
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

    if (!pb_decode(&spend_stream, penumbra_core_component_shielded_pool_v1_SpendPlan_fields, &spend_plan)) {
        return parser_spend_plan_error;
    }

    output->note.value.amount.lo = spend_plan.note.value.amount.lo;
    output->note.value.amount.hi = spend_plan.note.value.amount.hi;
    output->position = spend_plan.position;

    return parser_ok;
}

parser_error_t spend_getNumItems(const parser_context_t *ctx, uint8_t *num_items) {
    UNUSED(ctx);
    // from spends we display only two items:
    // - Spend 100 USDC
    // - From Main Account
    *num_items = 2;
    return parser_ok;
}

parser_error_t spend_getItem(const parser_context_t *ctx, const spend_plan_t *spend,
                             uint8_t displayIdx, char *outKey, uint16_t outKeyLen,
                             char *outVal, uint16_t outValLen, uint8_t pageIdx,
                             uint8_t *pageCount) {

    parser_error_t err = parser_no_data;
    if (spend == NULL || outKey == NULL || outVal == NULL || outKeyLen == 0 || outValLen == 0) {
        return err;
    }


    switch ( displayIdx ) {
        case 0:
            snprintf(outKey, outKeyLen, "Spend");
            return printValue(ctx, &spend->note.value.amount, &spend->note.value.asset_id.inner, outVal, outValLen);
            break;
        case 1:
            snprintf(outKey, outKeyLen, "From");
            snprintf(outVal, outValLen, "Main Account");
            break;
        default:
            return parser_no_data;
    }
    return parser_ok;

}
