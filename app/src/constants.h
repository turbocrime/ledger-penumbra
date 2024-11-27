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
#define U128_STR_MAX_LEN 40

// raw address len before encoding
#define ADDRESS_LEN_BYTES 80
// https://protocol.penumbra.zone/main/addresses_keys/addresses.html#short-address-form
#define SHORT_ADDRESS_VISIBLE_CHARS 24
#define ELLIPSIS "..."

// Common BECH32m constants
#define CHECKSUM_LENGTH 8
#define BECH32_BITS_PER_CHAR 5
#define BITS_PER_BYTE 8
#define BECH32_SEPARATOR "1"
#define SEPARATOR_LENGTH 1

// Some defines for address and asset encoding
#define ADDR_BECH32_PREFIX "penumbra"
// #define FIXED_ADDR_PREFIX ADDR_BECH32_PREFIX BECH32_SEPARATOR
#define ASSET_BECH32_PREFIX "passet"
#define ASSET_ID_LEN 32
// HRP length + 1 (separator) + 52 (data) + 6 (checksum) + 1 (null terminator)
// 6 + 1 + 52 + 6 + 2 = 67
#define ENCODED_ASSET_SIZE (strlen(ASSET_BECH32_PREFIX) + ((ASSET_ID_LEN * BITS_PER_BYTE + BECH32_BITS_PER_CHAR - 1) / BECH32_BITS_PER_CHAR) + CHECKSUM_LENGTH + 1)

#define ENCODED_DATA_LENGTH \
    (((ADDRESS_LEN_BYTES + CHECKSUM_LENGTH) * BITS_PER_BYTE + BECH32_BITS_PER_CHAR - 1) / BECH32_BITS_PER_CHAR)

#define ENCODED_ADDR_LEN (sizeof(ADDR_BECH32_PREFIX) + SEPARATOR_LENGTH + ENCODED_DATA_LENGTH)

#define ENCODED_ADDR_BUFFER_SIZE (ENCODED_ADDR_LEN + 2)


// MEMO transaction constants
#define MEMO_CIPHERTEXT_LEN_BYTES 528

// This is the `MEMO_CIPHERTEXT_LEN_BYTES` - MAC size (16 bytes).
#define MEMO_LEN_BYTES 512

// This is the largest text length we can support
#define MAX_TEXT_LEN MEMO_LEN_BYTES - ADDRESS_LEN_BYTES
