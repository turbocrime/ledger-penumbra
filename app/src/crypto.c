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

#include "crypto.h"

#include "coin.h"
#include "crypto_helper.h"
#include "cx.h"
#include "keys_def.h"
#include "parser_interface.h"
#include "zxformat.h"
#include "zxmacros.h"

uint32_t hdPath[HDPATH_LEN_DEFAULT];

__Z_INLINE zxerr_t copyKeys(keys_t *keys, key_kind_e req_type, uint8_t *output, uint16_t len, uint16_t *cmdResponseLen) {
    if (keys == NULL || output == NULL) {
        return zxerr_no_data;
    }

    switch (req_type) {
        case Address:
            if (len < ADDRESS_LEN_BYTES) {
                return zxerr_buffer_too_small;
            }
            memcpy(output, keys->address, ADDRESS_LEN_BYTES);
            *cmdResponseLen = ADDRESS_LEN_BYTES;
            break;

        case Fvk:
            *cmdResponseLen = 2 * KEY_LEN;

            if (len < *cmdResponseLen) {
                *cmdResponseLen = 0;
                return zxerr_buffer_too_small;
            }
            memcpy(output, keys->fvk, *cmdResponseLen);
            break;

        default:
            return zxerr_invalid_crypto_settings;
    }

    return zxerr_ok;
}

__Z_INLINE zxerr_t computeSpendKey(keys_t *keys) {
    if (keys == NULL) {
        return zxerr_no_data;
    }
    zxerr_t error = zxerr_invalid_crypto_settings;

    // Generate spending key
    uint8_t privateKeyData[SK_LEN_25519] = {0};
    CATCH_CXERROR(os_derive_bip32_no_throw(CX_CURVE_256K1, hdPath, HDPATH_LEN_DEFAULT, privateKeyData, NULL));

    memcpy(keys->skb, privateKeyData, sizeof(keys->skb));
    // if we reach this point no errors occurred
    error = zxerr_ok;

catch_cx_error:
    MEMZERO(&keys, sizeof(keys));
    MEMZERO(privateKeyData, sizeof(privateKeyData));

    return error;
}

zxerr_t crypto_fillKeys(uint8_t *output, uint16_t len, uint16_t *cmdResponseLen) {
    zemu_log("Crypto_fillKeys\n");

    keys_t keys = {0};
    zxerr_t error = zxerr_invalid_crypto_settings;

    if (output == NULL || len < sizeof(keys.fvk)) {
        return error;
    }

    // Compute seed
    CATCH_ZX_ERROR(computeSpendKey(&keys));

    // use seed to compute viewieng keys
    CATCH_ZX_ERROR(compute_keys(&keys));

    // Copy keys
    CATCH_ZX_ERROR(copyKeys(&keys, Fvk, output, len, cmdResponseLen));

    error = zxerr_ok;

catch_zx_error:
    MEMZERO(&keys, sizeof(keys));

    return error;
}

zxerr_t crypto_fillAddress(uint8_t *buffer, uint16_t bufferLen, uint16_t *cmdResponseLen, uint32_t account,
                           uint8_t *randomizer) {
    zemu_log("crypto_fillAddress\n");
    check_app_canary();

    keys_t keys = {0};
    zxerr_t error = zxerr_invalid_crypto_settings;

    if (buffer == NULL || cmdResponseLen == NULL) {
        return zxerr_invalid_crypto_settings;
    }

    MEMZERO(buffer, bufferLen);

    CATCH_ZX_ERROR(computeSpendKey(&keys));

    CATCH_ZX_ERROR(compute_address(&keys, account, randomizer));

    CATCH_ZX_ERROR(copyKeys(&keys, Address, buffer, bufferLen, cmdResponseLen));

    error = zxerr_ok;

catch_zx_error:
    MEMZERO(&keys, sizeof(keys));
    return error;
}

zxerr_t crypto_sign(parser_tx_t *tx_obj, uint8_t *signature, uint16_t signatureMaxlen) {
    if (signature == NULL || tx_obj == NULL || signatureMaxlen < EFFECT_HASH_LEN) {
        return zxerr_invalid_crypto_settings;
    }

    keys_t keys = {0};
    zxerr_t error = zxerr_invalid_crypto_settings;


    // compute parameters hash
    CATCH_ZX_ERROR(compute_parameters_hash(&tx_obj->parameters_plan.data_bytes, &tx_obj->plan.parameters_hash));

    // compute spend key
    CATCH_ZX_ERROR(computeSpendKey(&keys));

    // compute action hashes
    for (uint16_t i = 0; i < tx_obj->plan.actions.qty; i++) {
        CATCH_ZX_ERROR(compute_action_hash(&tx_obj->actions_plan[i], &keys.skb, &tx_obj->plan.memo.key,
                                           &tx_obj->plan.actions.hashes[i]));
    }

    // compute effect hash
    CATCH_ZX_ERROR(compute_effect_hash(&tx_obj->plan, tx_obj->effect_hash, sizeof(tx_obj->effect_hash)));

    MEMCPY(signature, tx_obj->effect_hash, EFFECT_HASH_LEN);

    return zxerr_ok;

catch_zx_error:
    MEMZERO(&keys, sizeof(keys));
    MEMZERO(signature, signatureMaxlen);

    if (error != zxerr_ok) {
        MEMZERO(signature, signatureMaxlen);
    }

    return error;
}
