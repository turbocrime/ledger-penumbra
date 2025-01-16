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
#include "undelegate_claim.h"

#include "note.h"
#include "parser_pb_utils.h"
#include "rslib.h"
#include "ui_utils.h"
#include "zxformat.h"

parser_error_t decode_undelegate_claim_plan(const bytes_t *data, undelegate_claim_plan_t *undelegate_claim_claim) {
    penumbra_core_component_stake_v1_UndelegateClaimPlan undelegate_claim_plan =
        penumbra_core_component_stake_v1_UndelegateClaimPlan_init_default;

    pb_istream_t stream = pb_istream_from_buffer(data->ptr, data->len);
    CHECK_APP_CANARY()

    // Set up fixed size fields
    fixed_size_field_t validator_identity_arg, penalty_arg, balance_blinding, proof_blinding_r, proof_blinding_s;
    setup_decode_fixed_field(&undelegate_claim_plan.validator_identity.ik, &validator_identity_arg,
                             &undelegate_claim_claim->validator_identity.ik, 32);
    setup_decode_fixed_field(&undelegate_claim_plan.penalty.inner, &penalty_arg, &undelegate_claim_claim->penalty.inner, 32);
    setup_decode_fixed_field(&undelegate_claim_plan.balance_blinding, &balance_blinding,
                             &undelegate_claim_claim->balance_blinding, 32);
    setup_decode_fixed_field(&undelegate_claim_plan.proof_blinding_r, &proof_blinding_r,
                             &undelegate_claim_claim->proof_blinding_r, 32);
    setup_decode_fixed_field(&undelegate_claim_plan.proof_blinding_s, &proof_blinding_s,
                             &undelegate_claim_claim->proof_blinding_s, 32);

    if (!pb_decode(&stream, penumbra_core_component_stake_v1_UndelegateClaimPlan_fields, &undelegate_claim_plan)) {
        return parser_undelegate_plan_error;
    }

    undelegate_claim_claim->has_validator_identity = undelegate_claim_plan.has_validator_identity;
    undelegate_claim_claim->start_epoch_index = undelegate_claim_plan.start_epoch_index;
    undelegate_claim_claim->has_penalty = undelegate_claim_plan.has_penalty;
    if (undelegate_claim_plan.has_unbonding_amount) {
        undelegate_claim_claim->unbonding_amount.lo = undelegate_claim_plan.unbonding_amount.lo;
        undelegate_claim_claim->unbonding_amount.hi = undelegate_claim_plan.unbonding_amount.hi;
    }
    undelegate_claim_claim->unbonding_start_height = undelegate_claim_plan.unbonding_start_height;

    return parser_ok;
}

parser_error_t undelegate_claim_getNumItems(const parser_context_t *ctx, uint8_t *num_items) {
    UNUSED(ctx);
    *num_items = 1;
    return parser_ok;
}

parser_error_t undelegate_claim_getItem(const parser_context_t *ctx, const undelegate_claim_plan_t *undelegate,
                                        uint8_t actionIdx, char *outKey, uint16_t outKeyLen, char *outVal,
                                        uint16_t outValLen, uint8_t pageIdx, uint8_t *pageCount) {
    parser_error_t err = parser_no_data;
    if (undelegate == NULL || outKey == NULL || outVal == NULL || outKeyLen == 0 || outValLen == 0) {
        return err;
    }

    char bufferUI[UNDELEGATE_DISPLAY_MAX_LEN] = {0};

    snprintf(outKey, outKeyLen, "Action_%d", actionIdx);
    CHECK_ERROR(undelegate_claim_printValue(ctx, undelegate, bufferUI, sizeof(bufferUI)));
    pageString(outVal, outValLen, bufferUI, pageIdx, pageCount);

    return parser_ok;
}

parser_error_t undelegate_claim_printValue(const parser_context_t *ctx, const undelegate_claim_plan_t *undelegate,
                                           char *outVal, uint16_t outValLen) {
    if (ctx == NULL || undelegate == NULL || outVal == NULL) {
        return parser_no_data;
    }

    if (outValLen < UNDELEGATE_DISPLAY_MAX_LEN) {
        return parser_unexpected_buffer_end;
    }

    MEMZERO(outVal, outValLen);

    // add action title
    snprintf(outVal, outValLen, "UndelegateClaim Value ");
    uint16_t written_value = strlen(outVal);

    // add validator identity
    uint8_t validator_identity_bytes[80] = {0};
    CHECK_ERROR(encodeIdentityKey(undelegate->validator_identity.ik.ptr, undelegate->validator_identity.ik.len,
                                  (char *)validator_identity_bytes, sizeof(validator_identity_bytes)));

    // add delegate amount
    uint8_t metadata_buffer[150] = {0};
    snprintf((char *)metadata_buffer, sizeof(metadata_buffer), "udelegation_%s", validator_identity_bytes);
    bytes_t metadata = {.ptr = metadata_buffer, .len = strlen((char *)metadata_buffer)};
    uint8_t asset_id_bytes[ASSET_ID_LEN] = {0};
    rs_get_asset_id_from_metadata(&metadata, asset_id_bytes, ASSET_ID_LEN);

    // add unbonded amount
    snprintf((char *)metadata_buffer, sizeof(metadata_buffer), "uunbonding_start_at_%llu_%s",
             undelegate->unbonding_start_height, validator_identity_bytes);
    metadata.len = strlen((char *)metadata_buffer);
    rs_get_asset_id_from_metadata(&metadata, asset_id_bytes, ASSET_ID_LEN);

    value_t local_unbonded_amount = {.amount = undelegate->unbonding_amount,
                                     .asset_id.inner = {.ptr = asset_id_bytes, .len = ASSET_ID_LEN},
                                     .has_amount = true,
                                     .has_asset_id = true};
    CHECK_ERROR(printValue(ctx, &local_unbonded_amount, &ctx->tx_obj->parameters_plan.chain_id, outVal + written_value,
                           outValLen - written_value));
    return parser_ok;
}
