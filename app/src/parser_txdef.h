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

#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>
#include <stdint.h>

#define MEMO_KEY_SIZE 32
#define MEMO_ADDRESS_INNER_SIZE 80
#define DETECTION_DATA_SIZE 122
#define RSEED_SIZE 32

// TODO: check size
#define DETECTION_DATA_QTY 16
#define ACTIONS_QTY 16

#define ASSET_ID_LEN 32
#define RSEED_LEN 32

typedef struct {
    const uint8_t *ptr;
    uint16_t len;
} Bytes_t;

typedef struct {
    uint64_t lo;
    uint64_t hi;
} amount_t;

typedef struct {
    Bytes_t inner;
} asset_id_t;

typedef struct {
    amount_t amount;
    asset_id_t asset_id;
} value_t;

typedef struct {
    Bytes_t inner;
    // Field bellow is a sort of optional
    // and is a shortcut for the case address is already
    // bech32m encoded
    Bytes_t alt_bech32m;
} address_plan_t;

typedef struct {
    value_t value;
    Bytes_t rseed;
    address_plan_t address;
} note_t;

typedef struct {
    note_t note;
    uint64_t position;
    Bytes_t randomizer;
    Bytes_t value_blinding;
    Bytes_t proof_blinding_r;
    Bytes_t proof_blinding_s;
} spend_plan_t;

typedef struct {
    Bytes_t parameters;
} transaction_parameters_t;

typedef struct {
    address_plan_t return_address;
    Bytes_t text;
} memo_plain_text_t;

typedef struct {
    memo_plain_text_t plaintext;
    Bytes_t key;
} memo_plan_t;

typedef struct {
    address_plan_t address;
    Bytes_t rseed;
    uint64_t precision_bits;
} clue_plan_t;

typedef struct {
    clue_plan_t clue_plans[DETECTION_DATA_QTY];
} detection_data_t;

typedef struct {
    uint8_t action_type;
    Bytes_t action;
} action_t;

typedef struct {
    action_t actions[ACTIONS_QTY];
    transaction_parameters_t transaction_parameters;
    memo_plan_t memo;
    detection_data_t detection_data;
} transaction_plan_t;

typedef struct {
    transaction_plan_t plan;
} parser_tx_t;

#ifdef __cplusplus
}
#endif
