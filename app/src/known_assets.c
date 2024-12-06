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

#include "known_assets.h"

#include <zxmacros.h>

// TODO: A place holder for known assets in penumbra
// bellow dummy values
static const asset_info_t supported_assets[] = {
    {STAKING_TOKEN_ASSET_ID_BYTES,
     "penumbra",
     "Penumbra",
     6},
};

const asset_info_t *asset_info_from_table(const uint8_t asset_id[ASSET_ID_LEN]) {
    unsigned int i;
    unsigned int info_len = sizeof(supported_assets) / sizeof(asset_info_t);
    for (i = 0; i < info_len; i++) {
        if (MEMCMP(supported_assets[i].asset_id, asset_id, ASSET_ID_LEN) == 0) {
            return &supported_assets[i];
        }
    }
    return NULL;
}
