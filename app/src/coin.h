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

#ifdef __cplusplus
extern "C" {
#endif

#define CLA 0x80

// according to penumbra docs:
// m/44'/6532'/0'
#define HDPATH_LEN_DEFAULT 3
#define HDPATH_0_DEFAULT (0x80000000u | 0x2c)    // 44
#define HDPATH_1_DEFAULT (0x80000000u | 0x1984)  // 6532
#define HDPATH_2_DEFAULT (0x80000000u | 0u)      // 0

#define SECP256K1_PK_LEN 65u

#define PK_LEN_25519 32u
#define SK_LEN_25519 64u
#define EFFECT_HASH_LEN 64u

#define COIN_AMOUNT_DECIMAL_PLACES 6
#define COIN_TICKER "penumbra"

#define MENU_MAIN_APP_LINE1 "Penumbra"
#define MENU_MAIN_APP_LINE2 "Ready"
#define MENU_MAIN_APP_LINE2_SECRET "???"
#define APPVERSION_LINE1 "Penumbra"
#define APPVERSION_LINE2 "v" APPVERSION

// Custom apdu instructions
#define INS_GET_FVK     0x03
#define INS_TX_METADATA 0x04

typedef enum {
    Address = 0,
    Fvk,
} key_kind_e;

#ifdef __cplusplus
}
#endif
