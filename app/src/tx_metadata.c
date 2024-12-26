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

#include "tx_metadata.h"

#include <zxmacros.h>

#include "constants.h"
#include "parser_txdef.h"
#include "rslib.h"

parser_error_t metadata_parse(const uint8_t *data, size_t dataLen, tx_metadata_t *metadata, uint8_t metadataLen) {
    if (metadata == NULL) {
        return parser_unexpected_error;
    }

    MEMZERO(metadata, sizeof(tx_metadata_t) * metadataLen);

    // Check that dataLen is at least 1 to read the number of metadata strings
    if (dataLen < 1) {
        return parser_unexpected_buffer_end;
    }

    // Read the number of metadata strings
    uint8_t num_strings = data[0];

    // Verify that the number of strings matches metadataLen
    if (num_strings >= metadataLen) {
        return parser_unexpected_number_items;
    }

    size_t data_offset = 1;  // Start after the num_strings byte

    // Iterate over each metadata string
    for (uint8_t i = 0; i < num_strings; i++) {
        // Check that there is at least one byte left for the length
        if (data_offset >= dataLen) {
            return parser_unexpected_buffer_end;
        }

        // Read the length of the current string
        uint8_t len = data[data_offset];
        data_offset += 1;

        // Validate the length to be within
        // the bounds of the denomination length minus
        // the null terminator character
        if (len > MAX_DENOM_LEN - 1) {
            return parser_value_out_of_range;
        }

        // Check that there are enough bytes left for the string data
        if (data_offset + len > dataLen) {
            return parser_unexpected_buffer_end;
        }

        // Copy the string data into the metadata array
        MEMCPY(metadata[i].denom, &data[data_offset], len);
        metadata[i].len = len;

        data_offset += len;
    }

    // Optional: Check if there are extra bytes left in the data buffer
    if (data_offset != dataLen) {
        // There are extra bytes in data, which might be unexpected
        return parser_unexpected_value;
    }

    return parser_ok;
}

parser_error_t metadata_toAssetId(const tx_metadata_t *metadata, uint8_t *asset, uint16_t asset_len) {
    bytes_t data = {0};
    data.ptr = (uint8_t *)&metadata->denom[0];
    data.len = metadata->len;

    CHECK_ERROR(rs_get_asset_id_from_metadata(&data, asset, asset_len));

    return parser_ok;
}

uint8_t metadata_getDenom(const tx_metadata_t *metadata, uint8_t metadataLen, const bytes_t *asset, char *denom,
                          uint8_t len) {
    if (metadataLen == 0 || metadata == NULL || asset == NULL || asset->len == 0 || denom == NULL || len == 0) {
        return 0;
    }

    uint8_t computed_asset[ASSET_ID_LEN] = {0};
    const tx_metadata_t *found = NULL;

    for (uint8_t i = 0; i < metadataLen; ++i) {
        found = &metadata[i];
        CHECK_ERROR(metadata_toAssetId(found, computed_asset, ASSET_ID_LEN));

        if (MEMCMP(computed_asset, asset->ptr, ASSET_ID_LEN) == 0) {
            if (found->len <= len) {
                MEMCPY(denom, found->denom, found->len);
                return found->len;
            }
        }
    }
    return 0;  // Asset not found
}
