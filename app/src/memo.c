/*******************************************************************************
 *  (c) 2018 - 2024 Zondax AG
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
#include "pb_common.h"
#include "ui_utils.h"
#include "zxformat.h"

parser_error_t memo_getNumItems(const parser_context_t *ctx, uint8_t *num_items) {
    UNUSED(ctx);
    if (ctx->tx_obj->plan.has_memo) {
        *num_items = 2;
    } else {
        *num_items = 0;
    }
    return parser_ok;
}

parser_error_t memo_getItem(const parser_context_t *ctx, uint8_t displayIdx, char *outKey, uint16_t outKeyLen, char *outVal,
                            uint16_t outValLen, uint8_t pageIdx, uint8_t *pageCount) {
    parser_error_t err = parser_no_data;
    if (ctx == NULL || outKey == NULL || outVal == NULL || outKeyLen == 0 || outValLen == 0) {
        return err;
    }

    char short_address[100] = {0};
    switch (displayIdx) {
        case 0:
            if (ctx->tx_obj->plan.has_memo) {
                snprintf(outKey, outKeyLen, "Memo Sender Address");
                CHECK_ERROR(printTxAddress(&ctx->tx_obj->plan.memo.plaintext.return_address.inner, short_address,
                                           sizeof(short_address)));
                pageString(outVal, outValLen, (char *)short_address, pageIdx, pageCount);
            } else {
                snprintf(outKey, outKeyLen, "Memo");
                snprintf(outVal, outValLen, "None");
            }
            return parser_ok;
        case 1:
            if (!ctx->tx_obj->plan.has_memo) {
                return parser_no_data;
            }
            snprintf(outKey, outKeyLen, "Memo Text");
            pageStringExt(outVal, outValLen, (char *)ctx->tx_obj->plan.memo.plaintext.text.ptr,
                          ctx->tx_obj->plan.memo.plaintext.text.len, pageIdx, pageCount);
            return parser_ok;
        default:
            return parser_no_data;
    }
    return parser_ok;
}
