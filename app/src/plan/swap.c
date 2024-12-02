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

parser_error_t decode_swap_plan(const bytes_t *data, swap_plan_t *swap) {
    penumbra_core_component_dex_v1_SwapPlan swap_plan = penumbra_core_component_dex_v1_SwapPlan_init_default;

    pb_istream_t swap_stream = pb_istream_from_buffer(data->ptr, data->len);
    CHECK_APP_CANARY()

    // Set up fixed size fields
    fixed_size_field_t fee_blinding_arg, proof_blinding_r_arg, proof_blinding_s_arg, asset_1_arg, asset_2_arg, fee_asset_id_arg, claim_address_arg, rseed_arg;
    setup_decode_fixed_field(&swap_plan.fee_blinding, &fee_blinding_arg, &swap->fee_blinding, 32);
    setup_decode_fixed_field(&swap_plan.proof_blinding_r, &proof_blinding_r_arg, &swap->proof_blinding_r, 32);
    setup_decode_fixed_field(&swap_plan.proof_blinding_s, &proof_blinding_s_arg, &swap->proof_blinding_s, 32);
    setup_decode_fixed_field(&swap_plan.swap_plaintext.trading_pair.asset_1.inner, &asset_1_arg, &swap->swap_plaintext.trading_pair.asset_1.inner, 32);
    setup_decode_fixed_field(&swap_plan.swap_plaintext.trading_pair.asset_2.inner, &asset_2_arg, &swap->swap_plaintext.trading_pair.asset_2.inner, 32);
    setup_decode_fixed_field(&swap_plan.swap_plaintext.claim_fee.asset_id.alt_bech32m, &fee_asset_id_arg, &swap->swap_plaintext.claim_fee.asset_id.inner, 32);
    setup_decode_fixed_field(&swap_plan.swap_plaintext.claim_address.inner, &claim_address_arg, &swap->swap_plaintext.claim_address.inner, 80);
    setup_decode_fixed_field(&swap_plan.swap_plaintext.rseed, &rseed_arg, &swap->swap_plaintext.rseed, 32);

    if (!pb_decode(&swap_stream, penumbra_core_component_dex_v1_SwapPlan_fields, &swap_plan)) {
        return parser_swap_plan_error;
    }

    swap->has_swap_plaintext = swap_plan.has_swap_plaintext;
    swap->swap_plaintext.has_trading_pair = swap_plan.swap_plaintext.has_trading_pair;
    if (swap->swap_plaintext.has_trading_pair) {
        swap->swap_plaintext.trading_pair.has_asset_1 = swap_plan.swap_plaintext.trading_pair.has_asset_1;
        swap->swap_plaintext.trading_pair.has_asset_2 = swap_plan.swap_plaintext.trading_pair.has_asset_2;
    }
    swap->swap_plaintext.has_delta_1_i = swap_plan.swap_plaintext.has_delta_1_i;
    if (swap->swap_plaintext.has_delta_1_i) {
        swap->swap_plaintext.delta_1_i.lo = swap_plan.swap_plaintext.delta_1_i.lo;
        swap->swap_plaintext.delta_1_i.hi = swap_plan.swap_plaintext.delta_1_i.hi;
    }
    swap->swap_plaintext.has_delta_2_i = swap_plan.swap_plaintext.has_delta_2_i;
    if (swap->swap_plaintext.has_delta_2_i) {
        swap->swap_plaintext.delta_2_i.lo = swap_plan.swap_plaintext.delta_2_i.lo;
        swap->swap_plaintext.delta_2_i.hi = swap_plan.swap_plaintext.delta_2_i.hi;
    }
    swap->swap_plaintext.has_claim_fee = swap_plan.swap_plaintext.has_claim_fee;
    if (swap->swap_plaintext.has_claim_fee) {
        swap->swap_plaintext.claim_fee.has_amount = swap_plan.swap_plaintext.claim_fee.has_amount;  
        if (swap->swap_plaintext.claim_fee.has_amount) {
            swap->swap_plaintext.claim_fee.amount.hi = swap_plan.swap_plaintext.claim_fee.amount.hi;
            swap->swap_plaintext.claim_fee.amount.lo = swap_plan.swap_plaintext.claim_fee.amount.lo;
        }
        swap->swap_plaintext.claim_fee.has_asset_id = swap_plan.swap_plaintext.claim_fee.has_asset_id;
    }
    swap->swap_plaintext.has_claim_address = swap_plan.swap_plaintext.has_claim_address;

    return parser_ok;
}
