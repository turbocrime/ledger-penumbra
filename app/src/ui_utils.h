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
#include "parser_common.h"

/**
 * @brief Encodes binary data into a Bech32 format string with a specified prefix.
 *
 * This function generates a Bech32-encoded string from the input data, appending a checksum.
 * It verifies that the input data has the expected length and that the output buffer
 * is large enough to accommodate the resulting string.
 *
 * @param[in] prefix        The human-readable part (HRP) of the Bech32 string.
 * @param[in] prefix_len    The length of the prefix, excluding the null terminator.
 * @param[in] data          Pointer to the binary data to encode.
 * @param[in] data_len      Length of the binary data to encode.
 * @param[in] expected_len  Expected length of the binary data (for validation).
 * @param[out] out          Buffer to store the resulting Bech32-encoded string.
 * @param[in] out_len       Length of the output buffer.
 *
 * @return parser_ok on success, or an appropriate error code:
 *         - `parser_unexpected_error` if encoding fails or the input data is NULL.
 *         - `parser_invalid_address` if the length of the input data does not match the expected length.
 *         - `parser_display_idx_out_of_range` if the output buffer is too small.
 */
parser_error_t printBech32Encoded(const char *prefix, uint16_t prefix_len, const uint8_t *data, uint16_t data_len,
                                  char *out, uint16_t out_len);

/**
 * Formats a raw Penumbra address into its canonical short form with Bech32 encoding.
 * Short form consists of: bech32 prefix + separator + first 24 chars + ellipsis
 * Example output: "penumbra19pj4ykqqzm9dzlq4dard8yrg…"
 *
 * @param[in]  address      Raw address bytes to be encoded
 * @param[in]  address_len  Length of the input address buffer
 * @param[out] out         Output buffer for the formatted address
 * @param[in]  out_len     Size of the output buffer
 *
 * @return parser_error_t   parser_ok on success, error code otherwise
 */
parser_error_t printTxAddress(const bytes_t *address, char *out, uint16_t out_len);
parser_error_t printShortAddress(const uint8_t *address, uint16_t address_len, char *out, uint16_t out_len);
parser_error_t encodeAddress(const uint8_t *address, uint16_t address_len, char *out, uint16_t out_len);
parser_error_t printAssetId(const uint8_t *asset, uint16_t asset_len, char *out, uint16_t out_len);
parser_error_t encodeIdentityKey(const uint8_t *identity_key, uint16_t identity_key_len, char *out, uint16_t out_len);
parser_error_t encodePositionId(const uint8_t *position_id, uint16_t position_id_len, char *out, uint16_t out_len);
parser_error_t encodeAuctionId(const uint8_t *auction_id, uint16_t auction_id_len, char *out, uint16_t out_len);

/**
 * Converts a 128-bit unsigned integer to its decimal string representation.
 * Output buffer will be null terminated and cleared using MEMZERO.
 *
 * @param[out] data    Output buffer for the resulting string
 * @param[in]  dataLen Size of the output buffer
 * @param[in]  high    Upper 64 bits of the 128-bit number
 * @param[in]  low     Lower 64 bits of the 128-bit number
 *
 * @return parser_error_t:
 *         - parser_ok on success
 *         - parser_no_data if data is NULL
 *         - parser_value_out_of_range if buffer too small (< U128_STR_MAX_LEN)
 */
parser_error_t uint128_to_str(char *data, int dataLen, uint64_t high, uint64_t low);
