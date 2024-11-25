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

#include <stdio.h>

#include "app_mode.h"
#include "coin.h"
#include "crypto.h"
#include "keys_def.h"
#include "parser_common.h"
#include "rslib.h"
#include "segwit_addr.h"
#include "zxerror.h"
#include "zxformat.h"
#include "zxmacros.h"

bool is_randomized = false;
uint32_t address_idx_account = 0;

#define BECH32_PREFIX "penumbra"

#define FIXED_PREFIX BECH32_PREFIX "1"
#define ADDRESS_NUM_CHARS_SHORT_FORM 24
#define NUM_CHARS_TO_DISPLAY 33

#define HRP_LENGTH (sizeof(FIXED_PREFIX) - 1)
#define RAW_ADDRESS_LENGTH 80
#define CHECKSUM_LENGTH 8
#define SEPARATOR_LENGTH 1

#define BECH32_BITS_PER_CHAR 5
#define BITS_PER_BYTE 8

#define ENCODED_DATA_LENGTH \
    (((RAW_ADDRESS_LENGTH + CHECKSUM_LENGTH) * BITS_PER_BYTE + BECH32_BITS_PER_CHAR - 1) / BECH32_BITS_PER_CHAR)

#define ENCODED_ADDR_LEN (HRP_LENGTH + SEPARATOR_LENGTH + ENCODED_DATA_LENGTH)

#define ENCODED_ADDR_BUFFER_SIZE (ENCODED_ADDR_LEN + 2)

zxerr_t addr_getNumItems(uint8_t *num_items) {
    zemu_log_stack("addr_getNumItems");
    // address, account and is_randomized flag
    *num_items = 3;
    return zxerr_ok;
}

zxerr_t addr_getItem(int8_t displayIdx, char *outKey, uint16_t outKeyLen, char *outVal, uint16_t outValLen, uint8_t pageIdx,
                     uint8_t *pageCount) {
    ZEMU_LOGF(50, "[addr_getItem] %d/%d\n", displayIdx, pageIdx)

    char encoded_addr[ENCODED_ADDR_BUFFER_SIZE] = {'\0'};

    switch (displayIdx) {
        case 0:
            snprintf(outKey, outKeyLen, "Address");

            int32_t ret = rs_bech32_encode((const uint8_t *)BECH32_PREFIX, sizeof(BECH32_PREFIX) - 1, G_io_apdu_buffer,
                                           ADDRESS_LEN_BYTES, (uint8_t *)encoded_addr, ENCODED_ADDR_BUFFER_SIZE);

            if (ret < 0) return zxerr_unknown;

            pageString(outVal, outValLen, encoded_addr, pageIdx, pageCount);
            return zxerr_ok;

        case 1: {
            snprintf(outKey, outKeyLen, "Account");
            char buffer[100] = {0};
            ZEMU_LOGF(50, "[Account****] %d\n", address_idx_account)

            const char *err = NULL;
            err = uint32_to_str(buffer, sizeof(buffer), address_idx_account);

            if (err != NULL) {
                return zxerr_unknown;
            }

            pageString(outVal, outValLen, buffer, pageIdx, pageCount);

            return zxerr_ok;
        }
        case 2: {
            snprintf(outKey, outKeyLen, "Randomized");
            const char *buffer = is_randomized ? "Yes" : "No";
            pageString(outVal, outValLen, buffer, pageIdx, pageCount);

            return zxerr_ok;
        }
        default:
            return zxerr_no_data;
    }
}
