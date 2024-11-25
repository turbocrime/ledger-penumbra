/*******************************************************************************
 *   (c) 2018 - 2023 Zondax AG
 *
 *  Licensed under the Apache License, Version 2.0 (the "License");
 *  you may not use this file except in compliance with the License.
G*  You may obtain a copy of the License at
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

#ifdef __cplusplus
extern "C" {
#endif

#include <sigutils.h>
#include <stdbool.h>

#include "coin.h"
#include "keys_def.h"
#include "parser_txdef.h"
#include "zxerror.h"

extern uint32_t hdPath[HDPATH_LEN_DEFAULT];

zxerr_t crypto_fillKeys(uint8_t *output, uint16_t len, uint16_t *cmdResponseLen);

zxerr_t crypto_fillAddress(uint8_t *buffer, uint16_t bufferLen, uint16_t *addrResponseLen, uint32_t account,
                           uint8_t *randomizer);

zxerr_t crypto_sign(parser_tx_t *tx_obj, uint8_t *signature, uint16_t signatureMaxlen);

zxerr_t crypto_extractSpendingKeyBytes(uint8_t *key_bytes, uint32_t key_bytes_len);

zxerr_t crypto_blake2b_512_init();

#ifdef __cplusplus
}
#endif
