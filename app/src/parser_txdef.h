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
#define CHAIN_ID_LEN 32

typedef struct {
    const uint8_t *ptr;
    uint16_t len;
} bytes_t;

typedef struct {
    uint64_t lo;
    uint64_t hi;
} amount_t;

typedef struct {
    bytes_t inner;
} asset_id_t;

typedef struct {
    amount_t amount;
    asset_id_t asset_id;
} value_t;

typedef struct {
    bytes_t inner;
    // Field bellow is a sort of optional
    // and is a shortcut for the case address is already
    // bech32m encoded
    bytes_t alt_bech32m;
} address_plan_t;

typedef struct {
    value_t value;
    bytes_t rseed;
    address_plan_t address;
} note_t;

typedef struct {
    bytes_t ik;
} identity_key_t;

typedef struct {
    uint64_t index;
    uint64_t start_height;
} epoch_t;

typedef struct {
    bool has_amount;
    amount_t amount;
    bool has_asset_id;
    asset_id_t asset_id;
} fee_t;

typedef struct {
    note_t note;
    uint64_t position;
    bytes_t randomizer;
    bytes_t value_blinding;
    bytes_t proof_blinding_r;
    bytes_t proof_blinding_s;
} spend_plan_t;

typedef struct {
    value_t value;
    address_plan_t dest_address;
    bytes_t rseed;
    bytes_t value_blinding;
    bytes_t proof_blinding_r;
    bytes_t proof_blinding_s;
} output_plan_t;

typedef struct {
    bool has_validator_identity;
    identity_key_t validator_identity;
    uint64_t epoch_index;
    bool has_unbonded_amount;
    amount_t unbonded_amount;
    bool has_delegation_amount;
    amount_t delegation_amount;
} delegate_plan_t;

typedef struct {
    bool has_validator_identity;
    identity_key_t validator_identity;
    uint64_t start_epoch_index;
    bool has_unbonded_amount;
    amount_t unbonded_amount;
    bool has_delegation_amount;
    amount_t delegation_amount;
    bool has_from_epoch;
    epoch_t from_epoch;
} undelegate_plan_t;
typedef struct {
    address_plan_t return_address;
    bytes_t text;
} memo_plain_text_t;

typedef struct {
    memo_plain_text_t plaintext;
    bytes_t key;
} memo_plan_t;

typedef struct {
    address_plan_t address;
    bytes_t rseed;
    uint64_t precision_bits;
} clue_plan_t;

typedef struct {
    clue_plan_t clue_plans[DETECTION_DATA_QTY];
} detection_data_t;

typedef struct {
    uint8_t action_type;
    bytes_t action_data;
    union {
        spend_plan_t spend;
        output_plan_t output;
        delegate_plan_t delegate;
        undelegate_plan_t undelegate;
    } action;
} action_t;

typedef uint8_t hash_t[64];
typedef struct {
    uint8_t qty;
    hash_t hashes[ACTIONS_QTY];
} actions_hash_t;

typedef struct {
    uint64_t expiry_height;
    bytes_t chain_id;
    bool has_fee;
    fee_t fee;
    bytes_t data_bytes;
} parameters_t;


typedef struct {
    actions_hash_t actions;
    hash_t parameters_hash;
    memo_plan_t memo;
    detection_data_t detection_data;
} transaction_plan_t;

typedef struct {
    transaction_plan_t plan;
    action_t actions_plan[ACTIONS_QTY];
    parameters_t parameters_plan;
    uint8_t effect_hash[64];
} parser_tx_t;

#ifdef __cplusplus
}
#endif
