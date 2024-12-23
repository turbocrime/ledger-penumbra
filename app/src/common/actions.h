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
#pragma once

#include <os_io_seproxyhal.h>
#include <stdint.h>

#include "addr.h"
#include "apdu_codes.h"
#include "coin.h"
#include "crypto.h"
#include "nv_signature.h"
#include "parser_interface.h"
#include "tx.h"
#include "zxerror.h"
#include "zxformat.h"

extern uint16_t cmdResponseLen;
extern uint32_t address_idx_account;

__Z_INLINE zxerr_t app_fill_address(address_index_t address_index) {
    check_app_canary();
    // Put data directly in the apdu buffer
    MEMZERO(G_io_apdu_buffer, IO_APDU_BUFFER_SIZE);

    cmdResponseLen = 0;

    zxerr_t error = crypto_fillAddress(G_io_apdu_buffer, IO_APDU_BUFFER_SIZE - 2, &cmdResponseLen, address_index.account,
                                       address_index.randomizer);

    if (error != zxerr_ok || cmdResponseLen == 0) {
        THROW(APDU_CODE_EXECUTION_ERROR);
    }

    // Set flag to show in case of requireConfirmation
    // that the address is indeed being randomized
    is_randomized = false;
    for (uint8_t i = 0; i < ADDR_RANDOMIZER_LEN; i++) {
        if (address_index.randomizer[i] != 0) {
            is_randomized = true;
            break;
        }
    }

    // Set the account in used to show it to the user
    // in case a review is enabled
    address_idx_account = address_index.account;

    return error;
}

__Z_INLINE zxerr_t app_fill_keys() {
    // Put data directly in the apdu buffer
    MEMZERO(G_io_apdu_buffer, IO_APDU_BUFFER_SIZE);

    cmdResponseLen = 0;

    zxerr_t err = crypto_fillKeys(G_io_apdu_buffer, IO_APDU_BUFFER_SIZE - 2, &cmdResponseLen);

    if (err != zxerr_ok || cmdResponseLen == 0) {
        THROW(APDU_CODE_EXECUTION_ERROR);
    }

    return zxerr_ok;
}

__Z_INLINE void app_sign() {
    MEMZERO(G_io_apdu_buffer, IO_APDU_BUFFER_SIZE);

    parser_tx_t *tx = tx_get_txObject();

    zxerr_t err = crypto_sign(tx, G_io_apdu_buffer, IO_APDU_BUFFER_SIZE - 3);

    check_app_canary();

    // |   64 bytes  |         2 bytes          |        2 bytes        |
    // | effect hash | spend auth signature qty | binding signature qty |
    if (err != zxerr_ok) {
        set_code(G_io_apdu_buffer, 0, APDU_CODE_SIGN_VERIFY_ERROR);
        io_exchange(CHANNEL_APDU | IO_RETURN_AFTER_TX, 2);
    } else {
        set_code(G_io_apdu_buffer, EFFECT_HASH_LEN + 2 * sizeof(uint16_t), APDU_CODE_OK);
        io_exchange(CHANNEL_APDU | IO_RETURN_AFTER_TX, EFFECT_HASH_LEN + 2 * sizeof(uint16_t) + 2);
    }
}

__Z_INLINE zxerr_t app_fill_signatures(uint16_t index, signature_type_t signature_type) {
    // Put data directly in the apdu buffer
    MEMZERO(G_io_apdu_buffer, IO_APDU_BUFFER_SIZE);

    cmdResponseLen = nv_get_signature(index, (signature_t *)G_io_apdu_buffer, signature_type);
    if (cmdResponseLen == 0) {
        THROW(APDU_CODE_EXECUTION_ERROR);
    }

    return zxerr_ok;
}

__Z_INLINE void app_reject() {
    MEMZERO(G_io_apdu_buffer, IO_APDU_BUFFER_SIZE);
    set_code(G_io_apdu_buffer, 0, APDU_CODE_COMMAND_NOT_ALLOWED);
    io_exchange(CHANNEL_APDU | IO_RETURN_AFTER_TX, 2);
}

__Z_INLINE void app_reply_address() {
    set_code(G_io_apdu_buffer, cmdResponseLen, APDU_CODE_OK);
    io_exchange(CHANNEL_APDU | IO_RETURN_AFTER_TX, cmdResponseLen + 2);
}

__Z_INLINE void app_reply_error() {
    set_code(G_io_apdu_buffer, 0, APDU_CODE_DATA_INVALID);
    io_exchange(CHANNEL_APDU | IO_RETURN_AFTER_TX, 2);
}
