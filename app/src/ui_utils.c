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

#include "ui_utils.h"

#include <stdio.h>

#include "constants.h"
#include "crypto_helper.h"
#include "parser_common.h"
#include "rslib.h"
#include "zxerror.h"
#include "zxformat.h"
#include "zxmacros.h"

parser_error_t printBech32Encoded(const char *prefix, uint16_t prefix_len, const uint8_t *data, uint16_t data_len,
                                  char *out, uint16_t out_len) {
    if (data == NULL) {
        return parser_unexpected_error;
    }

    // Check we have space for the null terminator
    if (out_len < prefix_len + ((data_len * 8 + 4) / 5) + CHECKSUM_LENGTH + 1) {
        return parser_unexpected_buffer_end;
    }

    MEMZERO(out, out_len);

    int32_t ret = rs_bech32_encode((const uint8_t *)prefix, prefix_len, data, data_len, (uint8_t *)out, out_len);

    if (ret < 0) {
        return parser_unexpected_error;
    }

    return parser_ok;
}

/// Prints an address
/// but a difference to the other formatting functions here, this method uses
/// the device keys to check if the passed address is visible or not
parser_error_t printTxAddress(const bytes_t *address, char *out, uint16_t out_len) {
    if (out == NULL || out_len == 0 || address == NULL) {
        return parser_no_data;
    }

    // Validate input length
    if (address->len != ADDRESS_LEN_BYTES) {
        return parser_invalid_address;
    }

    bool is_visible = false;
    uint32_t index = 0;
    CHECK_ERROR(rs_is_address_visible(address, &is_visible, &index));

    if (is_visible) {
        if (index == 0) {
            snprintf(out, out_len, "Main Account");
        } else {
            // We can use %d, because account is an uint32_t
            // otherwise u64_to_str or any other alternative
            // must be used
            snprintf(out, out_len, "Sub-account #%d", index);
        }
    } else {
        return printShortAddress(address->ptr, address->len, out, out_len);
    }

    return parser_ok;
}

parser_error_t encodeAddress(const uint8_t *address, uint16_t address_len, char *out, uint16_t out_len) {
    // Validate input length
    if (address_len != ADDRESS_LEN_BYTES) {
        return parser_invalid_address;
    }
    // printBech32Encoded(const char *prefix, uint16_t prefix_len, const uint8_t *data,
    //                                          uint16_t data_len, uint16_t expected_len,
    //                                          char *out, uint16_t out_len);
    return printBech32Encoded(ADDR_BECH32_PREFIX, sizeof(ADDR_BECH32_PREFIX) - 1, address, address_len, out, out_len);
}

parser_error_t printShortAddress(const uint8_t *address, uint16_t address_len, char *out, uint16_t out_len) {
    // First get the full address encoded
    char full_address[ENCODED_ADDR_BUFFER_SIZE] = {0};
    parser_error_t err = encodeAddress(address, address_len, full_address, (uint16_t)sizeof(full_address));
    if (err != parser_ok) {
        return err;
    }

    if (out_len < SHORT_ADDRESS_LEN) {
        return parser_unexpected_buffer_end;
    }

    uint16_t truncate_pos = SHORT_ADDRESS_LEN - sizeof(ELLIPSIS);
    MEMZERO(out, out_len);
    MEMCPY(out, full_address, truncate_pos);

    // Add ellipsis but omit the null character
    MEMCPY(out + truncate_pos, ELLIPSIS, sizeof(ELLIPSIS) - 1);

    return parser_ok;
}

parser_error_t printAssetId(const uint8_t *asset, uint16_t asset_len, char *out, uint16_t out_len) {
    if (asset_len != ASSET_ID_LEN) {
        return parser_unexpected_buffer_end;
    }

    return printBech32Encoded(ASSET_BECH32_PREFIX, sizeof(ASSET_BECH32_PREFIX) - 1, asset, asset_len, out, out_len);
}

parser_error_t encodeIdentityKey(const uint8_t *identity_key, uint16_t identity_key_len, char *out, uint16_t out_len) {
    // Validate input length
    if (identity_key_len != IDENTITY_KEY_LEN) {
        return parser_invalid_address;
    }
    return printBech32Encoded(IDENTITY_KEY_BECH32_PREFIX, sizeof(IDENTITY_KEY_BECH32_PREFIX) - 1, identity_key,
                              identity_key_len, out, out_len);
}

parser_error_t encodePositionId(const uint8_t *position_id, uint16_t position_id_len, char *out, uint16_t out_len) {
    // Validate input length
    if (position_id_len != POSITION_ID_LEN) {
        return parser_invalid_address;
    }
    return printBech32Encoded(POSITION_ID_BECH32_PREFIX, sizeof(POSITION_ID_BECH32_PREFIX) - 1, position_id,
                              position_id_len, out, out_len);
}

parser_error_t encodeAuctionId(const uint8_t *auction_id, uint16_t auction_id_len, char *out, uint16_t out_len) {
    // Validate input length
    if (auction_id_len != AUCTION_ID_LEN) {
        return parser_invalid_address;
    }
    return printBech32Encoded(AUCTION_ID_BECH32_PREFIX, sizeof(AUCTION_ID_BECH32_PREFIX) - 1, auction_id,
                              auction_id_len, out, out_len);
}

parser_error_t uint128_to_str(char *data, int dataLen, uint64_t high, uint64_t low) {
    if (data == NULL) return parser_no_data;
    if (dataLen < U128_STR_MAX_LEN) return parser_value_out_of_range;

    MEMZERO(data, dataLen);
    char *p = data;

    if (high == 0 && low == 0) {
        *(p++) = '0';
        return parser_ok;
    }

    uint64_t temp_high = high;
    uint64_t temp_low = low;

    while (temp_high != 0 || temp_low != 0) {
        if (p - data >= (dataLen - 1)) return parser_value_out_of_range;

        uint64_t quotient_high = 0;
        uint64_t quotient_low = 0;
        uint64_t remainder = 0;
        uint64_t current;

        current = temp_high;
        quotient_high = current / 10;
        remainder = current % 10;

        current = (remainder << 32) | (temp_low >> 32);
        uint64_t q = current / 10;
        remainder = current % 10;
        quotient_low = (q << 32);

        current = (remainder << 32) | (temp_low & 0xFFFFFFFF);
        q = current / 10;
        remainder = current % 10;
        quotient_low |= q;

        *(p++) = (char)('0' + remainder);
        temp_high = quotient_high;
        temp_low = quotient_low;
    }

    while (p > data) {
        p--;
        char z = *data;
        *data = *p;
        *p = z;
        data++;
    }

    return parser_ok;
}
