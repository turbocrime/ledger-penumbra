/*******************************************************************************
 *   (c) 2018 - 2023 Zondax AG
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
#include "parser_common.h"

#pragma once

#include <stdint.h>
#include "parser_common.h"

#ifdef __cplusplus
extern "C" {
#endif

parser_error_t printValue(const parser_context_t *ctx, const value_t *value, const bytes_t *chain_id,
                          char *outVal, uint16_t outValLen);

parser_error_t printFee(const parser_context_t *ctx, const value_t *value, const bytes_t *chain_id,
                          char *outVal, uint16_t outValLen);

parser_error_t tryPrintDenom(const parser_context_t *ctx, const value_t *value, const char *amount_str, char *outVal, uint16_t outValLen, bool *was_printed);
parser_error_t printFallback(const value_t *value, const char *amount_str, char *outVal, uint16_t outValLen);
parser_error_t printNumber(const char *amount, uint8_t decimalPlaces, const char *postfix, const char *prefix, char *outVal, uint16_t outValLen);
parser_error_t printAssetIdFromValue(const parser_context_t *ctx, const value_t *value, const bytes_t *chain_id, char *outVal, uint16_t outValLen);

#ifdef __cplusplus
}
#endif
