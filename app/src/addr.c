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
#include "ui_utils.h"

bool is_randomized = false;
uint32_t address_idx_account = 0;


zxerr_t addr_getNumItems(uint8_t *num_items) {
    zemu_log_stack("addr_getNumItems");
    // address, account and is_randomized flag
    *num_items = 3;
    return zxerr_ok;
}

zxerr_t addr_getItem(int8_t displayIdx, char *outKey, uint16_t outKeyLen, char *outVal, uint16_t outValLen, uint8_t pageIdx,
                     uint8_t *pageCount) {
    ZEMU_LOGF(50, "[addr_getItem] %d/%d\n", displayIdx, pageIdx)

    char encoded_addr[ENCODED_ADDR_BUFFER_SIZE + 1] = {'\0'};

    switch (displayIdx) {
        case 0: {
            snprintf(outKey, outKeyLen, "Address Index");
            if (address_idx_account == 0) {
                pageString(outVal, outValLen, "Main Account", pageIdx, pageCount);
                return zxerr_ok;
            } else {
                char buffer[200] = {0};
                snprintf(buffer, sizeof(buffer), "Sub-Account #%d\n", address_idx_account);

                pageString(outVal, outValLen, buffer, pageIdx, pageCount);
            }

            return zxerr_ok;
        }
        case 1:
            snprintf(outKey, outKeyLen, "Address");

            if (printShortAddress(G_io_apdu_buffer, ADDRESS_LEN_BYTES, encoded_addr, ENCODED_ADDR_BUFFER_SIZE) != parser_ok) {
                return zxerr_unknown;
            }

            pageString(outVal, outValLen, encoded_addr, pageIdx, pageCount);
            return zxerr_ok;

        case 2: {
            snprintf(outKey, outKeyLen, "IBC Deposit Address:");
            const char *buffer = is_randomized ? "Yes" : "No";
            pageString(outVal, outValLen, buffer, pageIdx, pageCount);

            return zxerr_ok;
        }
        default:
            return zxerr_no_data;
    }
}
