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
#pragma once

#include <zxmacros.h>

#include "parser_common.h"

#ifdef __cplusplus
extern "C" {
#endif

parser_error_t decode_undelegate_claim_plan(const bytes_t *data, undelegate_claim_plan_t *undelegate);
parser_error_t undelegate_claim_getNumItems(const parser_context_t *ctx, uint8_t *num_items);
parser_error_t undelegate_claim_getItem(const parser_context_t *ctx, const undelegate_claim_plan_t *undelegate,
                                        uint8_t actionIdx, char *outKey, uint16_t outKeyLen, char *outVal,
                                        uint16_t outValLen, uint8_t pageIdx, uint8_t *pageCount);
parser_error_t undelegate_claim_printValue(const parser_context_t *ctx, const undelegate_claim_plan_t *undelegate,
                                           char *outVal, uint16_t outValLen);

#ifdef __cplusplus
}
#endif
