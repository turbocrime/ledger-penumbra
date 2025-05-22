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
#include "crypto_helper.h"

#include <string.h>

#include "keys_def.h"
#include "rslib.h"
#include "zxformat.h"

zxerr_t compute_address(keys_t *keys, uint32_t account, uint8_t *randomizer) {
    if (keys == NULL) return zxerr_unknown;

    if (rs_compute_address(keys, account, randomizer) != parser_ok) {
        return zxerr_unknown;
    }

    return zxerr_ok;
}

zxerr_t compute_keys(keys_t *keys) {
    if (keys == NULL) return zxerr_unknown;

    if (rs_compute_keys(keys) != parser_ok) {
        return zxerr_unknown;
    }

    return zxerr_ok;
}
