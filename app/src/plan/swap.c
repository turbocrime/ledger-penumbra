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

#include "swap.h"

#include "note.h"
#include "parser_pb_utils.h"
#include "ui_utils.h"
#include "zxformat.h"

bool is_delta_empty(const amount_t *amount, const bool has_delta) {
    return !has_delta || (has_delta && amount->hi == 0 && amount->lo == 0);
}

parser_error_t decode_swap_plan(const bytes_t *data, swap_plan_t *swap) {
    penumbra_core_component_dex_v1_SwapPlan swap_plan = penumbra_core_component_dex_v1_SwapPlan_init_default;

    pb_istream_t stream = pb_istream_from_buffer(data->ptr, data->len);
    CHECK_APP_CANARY()

    // Set up fixed size fields
    fixed_size_field_t fee_blinding_arg, proof_blinding_r_arg, proof_blinding_s_arg, asset_1_arg, asset_2_arg,
        fee_asset_id_arg, claim_address_arg, rseed_arg;
    setup_decode_fixed_field(&swap_plan.fee_blinding, &fee_blinding_arg, &swap->fee_blinding, 32);
    setup_decode_fixed_field(&swap_plan.proof_blinding_r, &proof_blinding_r_arg, &swap->proof_blinding_r, 32);
    setup_decode_fixed_field(&swap_plan.proof_blinding_s, &proof_blinding_s_arg, &swap->proof_blinding_s, 32);
    setup_decode_fixed_field(&swap_plan.swap_plaintext.trading_pair.asset_1.inner, &asset_1_arg,
                             &swap->swap_plaintext.trading_pair.asset_1.inner, 32);
    setup_decode_fixed_field(&swap_plan.swap_plaintext.trading_pair.asset_2.inner, &asset_2_arg,
                             &swap->swap_plaintext.trading_pair.asset_2.inner, 32);
    setup_decode_fixed_field(&swap_plan.swap_plaintext.claim_fee.asset_id.alt_bech32m, &fee_asset_id_arg,
                             &swap->swap_plaintext.claim_fee.asset_id.inner, 32);
    setup_decode_fixed_field(&swap_plan.swap_plaintext.claim_address.inner, &claim_address_arg,
                             &swap->swap_plaintext.claim_address.inner, 80);
    setup_decode_fixed_field(&swap_plan.swap_plaintext.rseed, &rseed_arg, &swap->swap_plaintext.rseed, 32);

    if (!pb_decode(&stream, penumbra_core_component_dex_v1_SwapPlan_fields, &swap_plan)) {
        return parser_swap_plan_error;
    }

    swap->has_swap_plaintext = swap_plan.has_swap_plaintext;
    swap->swap_plaintext.has_trading_pair = swap_plan.swap_plaintext.has_trading_pair;
    if (swap->swap_plaintext.has_trading_pair) {
        swap->swap_plaintext.trading_pair.has_asset_1 = swap_plan.swap_plaintext.trading_pair.has_asset_1;
        swap->swap_plaintext.trading_pair.has_asset_2 = swap_plan.swap_plaintext.trading_pair.has_asset_2;
    }

    swap->swap_plaintext.has_delta_1_i = swap_plan.swap_plaintext.has_delta_1_i;
    swap->swap_plaintext.has_delta_2_i = swap_plan.swap_plaintext.has_delta_2_i;
    if (swap->swap_plaintext.has_delta_1_i) {
        swap->swap_plaintext.delta_1_i.lo = swap_plan.swap_plaintext.delta_1_i.lo;
        swap->swap_plaintext.delta_1_i.hi = swap_plan.swap_plaintext.delta_1_i.hi;
    }
    if (swap->swap_plaintext.has_delta_2_i) {
        swap->swap_plaintext.delta_2_i.lo = swap_plan.swap_plaintext.delta_2_i.lo;
        swap->swap_plaintext.delta_2_i.hi = swap_plan.swap_plaintext.delta_2_i.hi;
    }

    // check if both delta_1_i and delta_2_i are empty
    if (is_delta_empty(&swap->swap_plaintext.delta_1_i, swap->swap_plaintext.has_delta_1_i) &&
        is_delta_empty(&swap->swap_plaintext.delta_2_i, swap->swap_plaintext.has_delta_2_i)) {
        return parser_swap_plan_error;
    }

    // one delta must be nonzero
    if (!is_delta_empty(&swap->swap_plaintext.delta_1_i, swap->swap_plaintext.has_delta_1_i) &&
        !is_delta_empty(&swap->swap_plaintext.delta_2_i, swap->swap_plaintext.has_delta_2_i)) {
        return parser_swap_plan_error;
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

parser_error_t swap_getNumItems(const parser_context_t *ctx, uint8_t *num_items) {
    UNUSED(ctx);
    *num_items = 1;
    return parser_ok;
}

parser_error_t swap_getItem(const parser_context_t *ctx, const swap_plan_t *swap, uint8_t actionIdx, char *outKey,
                            uint16_t outKeyLen, char *outVal, uint16_t outValLen, uint8_t pageIdx, uint8_t *pageCount) {
    parser_error_t err = parser_no_data;
    if (swap == NULL || outKey == NULL || outVal == NULL || outKeyLen == 0 || outValLen == 0) {
        return err;
    }

    char bufferUI[SWAP_DISPLAY_MAX_LEN] = {0};

    snprintf(outKey, outKeyLen, "Action_%d", actionIdx);
    CHECK_ERROR(swap_printValue(ctx, swap, bufferUI, sizeof(bufferUI)));
    pageString(outVal, outValLen, bufferUI, pageIdx, pageCount);

    return parser_ok;
}

parser_error_t swap_printValue(const parser_context_t *ctx, const swap_plan_t *swap, char *outVal, uint16_t outValLen) {
    if (ctx == NULL || swap == NULL || outVal == NULL) {
        return parser_no_data;
    }

    if (outValLen < SWAP_DISPLAY_MAX_LEN) {
        return parser_unexpected_buffer_end;
    }

    MEMZERO(outVal, outValLen);

    // example: Output 100 USDC to penumbra1k0zzug62gpz60sejdvu9q7mq…

    // add action title
    snprintf(outVal, outValLen, "Swap ");
    uint16_t written_value = strlen(outVal);

    // add value
    value_t output_value = {0};
    value_t input_value = {0};
    if (!is_delta_empty(&swap->swap_plaintext.delta_1_i, swap->swap_plaintext.has_delta_1_i)) {
        input_value.amount.hi = swap->swap_plaintext.delta_1_i.hi;
        input_value.amount.lo = swap->swap_plaintext.delta_1_i.lo;
        input_value.asset_id.inner.ptr = swap->swap_plaintext.trading_pair.asset_1.inner.ptr;
        input_value.asset_id.inner.len = swap->swap_plaintext.trading_pair.asset_1.inner.len;

        output_value.amount.hi = 0;
        output_value.amount.lo = 0;
        output_value.asset_id.inner.ptr = swap->swap_plaintext.trading_pair.asset_2.inner.ptr;
        output_value.asset_id.inner.len = swap->swap_plaintext.trading_pair.asset_2.inner.len;
    } else {
        input_value.amount.hi = swap->swap_plaintext.delta_2_i.hi;
        input_value.amount.lo = swap->swap_plaintext.delta_2_i.lo;
        input_value.asset_id.inner.ptr = swap->swap_plaintext.trading_pair.asset_2.inner.ptr;
        input_value.asset_id.inner.len = swap->swap_plaintext.trading_pair.asset_2.inner.len;

        output_value.amount.hi = 0;
        output_value.amount.lo = 0;
        output_value.asset_id.inner.ptr = swap->swap_plaintext.trading_pair.asset_1.inner.ptr;
        output_value.asset_id.inner.len = swap->swap_plaintext.trading_pair.asset_1.inner.len;
    }
    input_value.has_asset_id = true;
    input_value.has_amount = true;
    output_value.has_asset_id = true;
    output_value.has_amount = true;

    // add "input"
    snprintf(outVal + written_value, outValLen - written_value, "Input ");
    written_value = strlen(outVal);

    CHECK_ERROR(printValue(ctx, &input_value, &ctx->tx_obj->parameters_plan.chain_id, true, outVal + written_value,
                           outValLen - written_value));
    written_value = strlen(outVal);

    snprintf(outVal + written_value, outValLen - written_value, " Output Asset ");
    written_value = strlen(outVal);

    CHECK_ERROR(printAssetIdFromValue(ctx, &input_value, &ctx->tx_obj->parameters_plan.chain_id, outVal + written_value,
                                      outValLen - written_value));
    written_value = strlen(outVal);

    snprintf(outVal + written_value, outValLen - written_value, " Claim Fee ");
    written_value = strlen(outVal);

    CHECK_ERROR(printFee(ctx, &swap->swap_plaintext.claim_fee, &ctx->tx_obj->parameters_plan.chain_id,
                         outVal + written_value, outValLen - written_value));

    return parser_ok;
}
