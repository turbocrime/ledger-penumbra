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
#include "nv_signature.h"
#include "rslib.h"

// TODO: Maybe move this to crypto_helper
#include "protobuf/penumbra/core/transaction/v1/transaction.pb.h"

uint32_t hdPath[HDPATH_LEN_DEFAULT];
full_viewing_key_t fvk_cached = {0};
bool fvk_cached_set = false;

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

    if (!fvk_cached_set) {
        // Compute seed
        CATCH_ZX_ERROR(computeSpendKey(&keys));

        // use seed to compute viewieng keys
        CATCH_ZX_ERROR(compute_keys(&keys));

        // Copy keys
        CATCH_ZX_ERROR(copyKeys(&keys, Fvk, output, len, cmdResponseLen));

        MEMCPY(fvk_cached, keys.fvk, FVK_LEN);

        fvk_cached_set = true;
    } else {
        MEMCPY(output, fvk_cached, FVK_LEN);
    }

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
    if (signature == NULL || tx_obj == NULL || signatureMaxlen < EFFECT_HASH_LEN + 2 * sizeof(uint16_t)) {
        return zxerr_invalid_crypto_settings;
    }

    keys_t keys = {0};
    nv_signature_init();

    zxerr_t error = zxerr_invalid_crypto_settings;

    // compute parameters hash
    CATCH_ZX_ERROR(compute_parameters_hash(&tx_obj->parameters_plan.data_bytes, &tx_obj->plan.parameters_hash));

    // compute spend key
    CATCH_ZX_ERROR(computeSpendKey(&keys));

    // compute action hashes
    for (uint16_t i = 0; i < tx_obj->plan.actions.qty; i++) {
        CATCH_ZX_ERROR(
            compute_action_hash(&tx_obj->actions_plan[i], &tx_obj->plan.memo.key, &tx_obj->plan.actions.hashes[i]));
    }

    // compute effect hash
    CATCH_ZX_ERROR(compute_effect_hash(&tx_obj->plan, tx_obj->effect_hash, sizeof(tx_obj->effect_hash)));

    // Similar to what is done in:
    // https://github.com/penumbra-zone/penumbra/blob/main/crates/core/transaction/src/plan/auth.rs#L12
    uint8_t spend_signature[64] = {0};
    bytes_t effect_hash = {.ptr = tx_obj->effect_hash, .len = 64};
    for (uint16_t i = 0; i < tx_obj->plan.actions.qty; i++) {
        if (tx_obj->actions_plan[i].action_type == penumbra_core_transaction_v1_ActionPlan_spend_tag){
            if (rs_sign_spend(&effect_hash, &tx_obj->actions_plan[i].action.spend.randomizer, &keys.skb, spend_signature, 64) != parser_ok) {
                return zxerr_invalid_crypto_settings;
            }

            // TODO:
            // Copy signature to flash either one by one
            // or by chunks.
            if (!nv_write_signature(spend_signature, Spend)) {
                return zxerr_buffer_too_small;
            }
        }
    }

    uint8_t *current_ptr = signature;
    MEMCPY(current_ptr, tx_obj->effect_hash, EFFECT_HASH_LEN);
    current_ptr += EFFECT_HASH_LEN;
    uint16_t spend_signatures = (uint16_t)nv_num_signatures(Spend);
    uint16_t delegator_signatures = 0;
    MEMCPY(current_ptr, &spend_signatures, sizeof(uint16_t));
    current_ptr += sizeof(uint16_t);

    MEMCPY(current_ptr, &delegator_signatures, sizeof(uint16_t));

    return zxerr_ok;

catch_zx_error:
    MEMZERO(signature, signatureMaxlen);

    if (error != zxerr_ok) {
        MEMZERO(signature, signatureMaxlen);
    }

    return error;
}
