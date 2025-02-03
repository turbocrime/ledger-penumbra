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
#include "undelegate.h"

#include "note.h"
#include "parser_pb_utils.h"
#include "rslib.h"
#include "ui_utils.h"
#include "zxformat.h"

parser_error_t decode_undelegate_plan(const bytes_t *data, undelegate_plan_t *undelegate) {
    penumbra_core_component_stake_v1_Undelegate undelegate_plan = penumbra_core_component_stake_v1_Undelegate_init_default;

    pb_istream_t stream = pb_istream_from_buffer(data->ptr, data->len);
    CHECK_APP_CANARY()

    // Set up fixed size fields
    fixed_size_field_t validator_identity_arg;
    setup_decode_fixed_field(&undelegate_plan.validator_identity.ik, &validator_identity_arg,
                             &undelegate->validator_identity.ik, 32);

    if (!pb_decode(&stream, penumbra_core_component_stake_v1_Undelegate_fields, &undelegate_plan)) {
        return parser_undelegate_plan_error;
    }

    undelegate->has_validator_identity = undelegate_plan.has_validator_identity;
    if (undelegate_plan.has_unbonded_amount) {
        undelegate->unbonded_amount.lo = undelegate_plan.unbonded_amount.lo;
        undelegate->unbonded_amount.hi = undelegate_plan.unbonded_amount.hi;
    }
    if (undelegate_plan.has_delegation_amount) {
        undelegate->delegation_amount.lo = undelegate_plan.delegation_amount.lo;
        undelegate->delegation_amount.hi = undelegate_plan.delegation_amount.hi;
    }
    if (undelegate_plan.has_from_epoch) {
        undelegate->from_epoch.index = undelegate_plan.from_epoch.index;
        undelegate->from_epoch.start_height = undelegate_plan.from_epoch.start_height;
    }

    return parser_ok;
}

parser_error_t undelegate_getNumItems(const parser_context_t *ctx, uint8_t *num_items) {
    UNUSED(ctx);
    *num_items = 1;
    return parser_ok;
}

parser_error_t undelegate_getItem(const parser_context_t *ctx, const undelegate_plan_t *undelegate, uint8_t actionIdx,
                                  char *outKey, uint16_t outKeyLen, char *outVal, uint16_t outValLen, uint8_t pageIdx,
                                  uint8_t *pageCount) {
    parser_error_t err = parser_no_data;
    if (undelegate == NULL || outKey == NULL || outVal == NULL || outKeyLen == 0 || outValLen == 0) {
        return err;
    }

    char bufferUI[UNDELEGATE_DISPLAY_MAX_LEN] = {0};

    snprintf(outKey, outKeyLen, "Action_%d", actionIdx);
    CHECK_ERROR(undelegate_printValue(ctx, undelegate, bufferUI, sizeof(bufferUI)));
    pageString(outVal, outValLen, bufferUI, pageIdx, pageCount);

    return parser_ok;
}

parser_error_t undelegate_printValue(const parser_context_t *ctx, const undelegate_plan_t *undelegate, char *outVal,
                                     uint16_t outValLen) {
    if (ctx == NULL || undelegate == NULL || outVal == NULL) {
        return parser_no_data;
    }

    if (outValLen < UNDELEGATE_DISPLAY_MAX_LEN) {
        return parser_unexpected_buffer_end;
    }

    MEMZERO(outVal, outValLen);

    // add action title
    snprintf(outVal, outValLen, "Undelegate From ");
    uint16_t written_value = strlen(outVal);

    // add validator identity
    uint8_t validator_identity_bytes[80] = {0};
    CHECK_ERROR(encodeIdentityKey(undelegate->validator_identity.ik.ptr, undelegate->validator_identity.ik.len,
                                  (char *)validator_identity_bytes, sizeof(validator_identity_bytes)));

    snprintf(outVal + written_value, outValLen - written_value, "%s", validator_identity_bytes);
    written_value = strlen(outVal);

    // add "Input"
    snprintf(outVal + written_value, outValLen - written_value, " Input ");
    written_value = strlen(outVal);

    // add delegate amount
    uint8_t metadata_buffer[150] = {0};
    snprintf((char *)metadata_buffer, sizeof(metadata_buffer), "udelegation_%s", validator_identity_bytes);
    bytes_t metadata = {.ptr = metadata_buffer, .len = strlen((char *)metadata_buffer)};
    uint8_t asset_id_bytes[ASSET_ID_LEN] = {0};
    rs_get_asset_id_from_metadata(&metadata, asset_id_bytes, ASSET_ID_LEN);

    value_t local_delegate_amount = {.amount = undelegate->delegation_amount,
                                     .asset_id.inner = {.ptr = asset_id_bytes, .len = ASSET_ID_LEN},
                                     .has_amount = true,
                                     .has_asset_id = true};
    CHECK_ERROR(printValue(ctx, &local_delegate_amount, &ctx->tx_obj->parameters_plan.chain_id, true, outVal + written_value,
                           outValLen - written_value));
    written_value = strlen(outVal);

    // add "Output"
    snprintf(outVal + written_value, outValLen - written_value, " Output ");
    written_value = strlen(outVal);

    // add unbonded amount
    snprintf((char *)metadata_buffer, sizeof(metadata_buffer), "uunbonding_start_at_");
    uint16_t written_value_metadata = strlen((char *)metadata_buffer);
    uint64_to_str((char *)metadata_buffer + written_value_metadata, sizeof(metadata_buffer) - written_value_metadata,
                  undelegate->from_epoch.index);
    written_value_metadata = strlen((char *)metadata_buffer);
    snprintf((char *)metadata_buffer + written_value_metadata, sizeof(metadata_buffer) - written_value_metadata, "_");
    written_value_metadata = strlen((char *)metadata_buffer);
    snprintf((char *)metadata_buffer + written_value_metadata, sizeof(metadata_buffer) - written_value_metadata, "%s",
             validator_identity_bytes);
    written_value_metadata = strlen((char *)metadata_buffer);
    metadata.len = written_value_metadata;
    rs_get_asset_id_from_metadata(&metadata, asset_id_bytes, ASSET_ID_LEN);

    value_t local_unbonded_amount = {.amount = undelegate->unbonded_amount,
                                     .asset_id.inner = {.ptr = asset_id_bytes, .len = ASSET_ID_LEN},
                                     .has_amount = true,
                                     .has_asset_id = true};
    CHECK_ERROR(printValue(ctx, &local_unbonded_amount, &ctx->tx_obj->parameters_plan.chain_id, true, outVal + written_value,
                           outValLen - written_value));
    return parser_ok;
}
