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
// plus null terminator
#define MAX_DENOM_LEN 120 + 1

// Constant to use to allocate a buffer on the stack
// to hold the formatting of a value_t type, following
// provided documentation, we choose the worst case, where
// a value contains an unknown token and we use a custom denom
// U128 + space + denom + null terminator
#define VALUE_DISPLAY_MAX_LEN (U128_STR_MAX_LEN + 1 + MAX_DENOM_LEN)  // = 162

#define SIGNATURE_LEN_BYTES 64

// raw address len before encoding
#define ADDRESS_LEN_BYTES 80
// https://protocol.penumbra.zone/main/addresses_keys/addresses.html#short-address-form
#define SHORT_ADDRESS_VISIBLE_CHARS 24
#define ELLIPSIS "…"

#define IDENTITY_KEY_BECH32_PREFIX "penumbravalid"
#define IDENTITY_KEY_LEN 32

#define POSITION_ID_BECH32_PREFIX "plpid"
#define POSITION_ID_LEN 32

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
#define ENCODED_ASSET_SIZE                                                                                              \
    (strlen(ASSET_BECH32_PREFIX) + ((ASSET_ID_LEN * BITS_PER_BYTE + BECH32_BITS_PER_CHAR - 1) / BECH32_BITS_PER_CHAR) + \
     CHECKSUM_LENGTH + 1)

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
// The number of metadata we can handle in RAM during
// transaction signing
#define MAX_TX_METADATA_LEN 5

// The staking token asset ID (upenumbra)
// Bech32m: passet1984fctenw8m2fpl8a9wzguzp7j34d7vravryuhft808nyt9fdggqxmanqm
#define STAKING_TOKEN_ASSET_ID_BYTES                                                                                      \
    {                                                                                                                     \
        0x29, 0xea, 0x9c, 0x2f, 0x33, 0x71, 0xf6, 0xa4, 0x87, 0xe7, 0xe9, 0x5c, 0x24, 0x70, 0x41, 0xf4, 0xa3, 0x56, 0xf9, \
            0x83, 0xeb, 0x06, 0x4e, 0x5d, 0x2b, 0x3b, 0xcf, 0x32, 0x2c, 0xa9, 0x6a, 0x10                                  \
    }

#define DEFAULT_CHAIN_ID "penumbra-1"

// Constant to use to allocate a buffer on the stack to hold the formatting of an output action
#define OUTPUT_DISPLAY_MAX_LEN \
    (VALUE_DISPLAY_MAX_LEN + SHORT_ADDRESS_VISIBLE_CHARS + sizeof(ELLIPSIS) + sizeof(ADDR_BECH32_PREFIX) + 6)  // = 202

// Constant to use to allocate a buffer on the stack to hold the formatting of an spend action
#define SPEND_DISPLAY_MAX_LEN \
    (VALUE_DISPLAY_MAX_LEN + SHORT_ADDRESS_VISIBLE_CHARS + sizeof(ELLIPSIS) + sizeof(ADDR_BECH32_PREFIX) + 10)  // = 202

// Constant to use to allocate a buffer on the stack to hold the formatting of an swap action
#define SWAP_DISPLAY_MAX_LEN (2 * VALUE_DISPLAY_MAX_LEN + SHORT_ADDRESS_VISIBLE_CHARS + sizeof(ELLIPSIS) + 6)  // = 355

// Constant to use to allocate a buffer on the stack to hold the formatting of an ics20 withdrawal action
#define ICS20_WITHDRAWAL_DISPLAY_MAX_LEN \
    (VALUE_DISPLAY_MAX_LEN + 300 + 36)  // = 498 -> 300 bytes for the channel and destination address

// Constant to use to allocate a buffer on the stack to hold the formatting of an delegate action
#define DELEGATE_DISPLAY_MAX_LEN (VALUE_DISPLAY_MAX_LEN + 92)  // = 254

// Constant to use to allocate a buffer on the stack to hold the formatting of an undelegate action
#define UNDELEGATE_DISPLAY_MAX_LEN (2 * VALUE_DISPLAY_MAX_LEN + 100)  // = 424

// Constant to use to allocate a buffer on the stack to hold the formatting of an position_open action
#define POSITION_OPEN_DISPLAY_MAX_LEN (2 * VALUE_DISPLAY_MAX_LEN + 110)  // = 434

// Constant to use to allocate a buffer on the stack to hold the formatting of an position_close action
#define POSITION_CLOSE_DISPLAY_MAX_LEN 100  // = 100

// Constant to use to allocate a buffer on the stack to hold the formatting of an position_withdraw action
#define POSITION_WITHDRAW_DISPLAY_MAX_LEN 140  // = 140

// Constant to use to allocate a buffer on the stack to hold the formatting of an dutch_auction_schedule action
#define DUTCH_AUCTION_SCHEDULE_DISPLAY_MAX_LEN (4 * VALUE_DISPLAY_MAX_LEN + 154)  // = 802
