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
#include "parser_impl.h"
#include "parser_interface.h"
#include "parser_pb_utils.h"
#include "pb_common.h"
#include "pb_decode.h"
#include "ui_utils.h"
#include "zxformat.h"

parser_error_t decode_parameters(const bytes_t *data,
                                 const penumbra_core_transaction_v1_TransactionParameters *transaction_parameters,
                                 parameters_t *parameters) {
    // get transaction parameters
    CHECK_ERROR(extract_data_from_tag(data, &parameters->data_bytes,
                                      penumbra_core_transaction_v1_TransactionPlan_transaction_parameters_tag));

    // copy parameters
    parameters->expiry_height = transaction_parameters->expiry_height;
    parameters->has_fee = transaction_parameters->has_fee;
    if (parameters->has_fee) {
        parameters->fee.has_amount = transaction_parameters->fee.has_amount;
        if (parameters->fee.has_amount) {
            parameters->fee.amount.lo = transaction_parameters->fee.amount.lo;
            parameters->fee.amount.hi = transaction_parameters->fee.amount.hi;
        }
        parameters->fee.has_asset_id = transaction_parameters->fee.has_asset_id;
    }

    return parser_ok;
}

parser_error_t parameters_getNumItems(const parser_context_t *ctx, uint8_t *num_items) {
    UNUSED(ctx);
    if (ctx->tx_obj->parameters_plan.expiry_height == 0) {
        *num_items = 2;
    } else {
        *num_items = 3;
    }
    return parser_ok;
}

parser_error_t parameters_getItem(const parser_context_t *ctx, uint8_t displayIdx, char *outKey, uint16_t outKeyLen,
                                  char *outVal, uint16_t outValLen, uint8_t pageIdx, uint8_t *pageCount) {
    parser_error_t err = parser_no_data;
    if (ctx == NULL || outKey == NULL || outVal == NULL || outKeyLen == 0 || outValLen == 0) {
        return err;
    }

    char bufferUI[VALUE_DISPLAY_MAX_LEN] = {0};
    switch (displayIdx) {
        case 0:
            snprintf(outKey, outKeyLen, "Chain ID");
            pageStringExt(outVal, outValLen, (char *)ctx->tx_obj->parameters_plan.chain_id.ptr,
                          ctx->tx_obj->parameters_plan.chain_id.len, pageIdx, pageCount);
            return parser_ok;
        case 1:
            if (ctx->tx_obj->parameters_plan.expiry_height == 0) {
                snprintf(outKey, outKeyLen, "Fee");
                CHECK_ERROR(printFee(ctx, &ctx->tx_obj->parameters_plan.fee, &ctx->tx_obj->parameters_plan.chain_id,
                                     bufferUI, sizeof(bufferUI)));
            } else {
                snprintf(outKey, outKeyLen, "Expiry Height");
                if (uint64_to_str(bufferUI, sizeof(bufferUI), ctx->tx_obj->parameters_plan.expiry_height) != NULL) {
                    return parser_unexpected_value;
                }
            }
            pageString(outVal, outValLen, bufferUI, pageIdx, pageCount);
            return parser_ok;
        case 2:
            snprintf(outKey, outKeyLen, "Fee");
            CHECK_ERROR(printFee(ctx, &ctx->tx_obj->parameters_plan.fee, &ctx->tx_obj->parameters_plan.chain_id, bufferUI,
                                 sizeof(bufferUI)));
            pageString(outVal, outValLen, bufferUI, pageIdx, pageCount);
            return parser_ok;
        default:
            return parser_no_data;
    }
    return parser_ok;
}