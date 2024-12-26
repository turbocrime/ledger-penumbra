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
#pragma once

#ifdef __cplusplus
extern "C" {
#endif

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include "keys_def.h"
#include "parser_common.h"
#include "zxerror.h"
#include "zxmacros.h"

#define ASSERT_CX_OK(CALL)                  \
    do {                                    \
        cx_err_t __cx_err = CALL;           \
        if (__cx_err != CX_OK) {            \
            return parser_unexpected_error; \
        }                                   \
    } while (0)

// catch zxerr_t errors
#define CATCH_ZX_ERROR(CALL)     \
    do {                         \
        error = CALL;            \
        if (error != zxerr_ok) { \
            goto catch_zx_error; \
        }                        \
    } while (0)

zxerr_t compute_address(keys_t *keys, uint32_t account, uint8_t *randomizer);
zxerr_t compute_keys(keys_t *keys);

#ifdef __cplusplus
}
#endif
