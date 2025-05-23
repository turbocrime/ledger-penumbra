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

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include "constants.h"

#define MEMO_KEY_SIZE 32
#define MEMO_ADDRESS_INNER_SIZE 80
#define DETECTION_DATA_SIZE 122
#define RSEED_SIZE 32

#define DETECTION_DATA_QTY 16
#define ACTIONS_QTY 16
#define MAX_CALLBACK_ARRAY_SIZE 5

#define ASSET_ID_LEN 32
#define RSEED_LEN 32
#define CHAIN_ID_LEN 32

#define MAX_SYMBOL_LEN 80
#define MAX_ASSET_NAME_LEN 120

typedef enum { VOTE_UNSPECIFIED = 0, VOTE_ABSTAIN = 1, VOTE_YES = 2, VOTE_NO = 3 } governance_vote_e;

typedef enum {
    POSITION_STATE_ENUM_UNSPECIFIED = 0,
    POSITION_STATE_ENUM_OPENED = 1,
    POSITION_STATE_ENUM_CLOSED = 2,
    POSITION_STATE_ENUM_WITHDRAWN = 3,
    POSITION_STATE_ENUM_CLAIMED = 4
} position_state_enum_t;

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
    bool has_amount;
    amount_t amount;
    bool has_asset_id;
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
    bool has_value;
    value_t value;
    bytes_t rseed;
    bool has_address;
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
    bytes_t inner;
} denom_t;

typedef struct {
    uint64_t revision_number;
    uint64_t revision_height;
} height_t;

typedef struct {
    bool has_asset_1;
    asset_id_t asset_1;
    bool has_asset_2;
    asset_id_t asset_2;
} trading_pair_t;

typedef struct {
    bytes_t inner;
} balance_commitment_t;

typedef struct {
    bytes_t inner;
} state_commitment_t;

typedef struct {
    bool has_commitment;
    state_commitment_t commitment;
    bytes_t encrypted_swap;
} swap_payload_t;

typedef struct {
    bool has_trading_pair;
    trading_pair_t trading_pair;
    bool has_delta_1_i;
    amount_t delta_1_i;
    bool has_delta_2_i;
    amount_t delta_2_i;
    bool has_claim_fee;
    value_t claim_fee;
    bool has_claim_address;
    address_plan_t claim_address;
    bytes_t rseed;
} swap_plaintext_t;

typedef struct {
    bytes_t inner;
} penalty_t;

typedef struct {
    governance_vote_e vote_enum;
} vote_t;

typedef struct {
    uint32_t fee;
    bool has_p;
    amount_t p;
    bool has_q;
    amount_t q;
} bare_trading_function_t;

typedef struct {
    bool has_component;
    bare_trading_function_t component;
    bool has_pair;
    trading_pair_t pair;
} trading_function_t;

typedef struct {
    position_state_enum_t state;
    uint64_t sequence;
} position_state_t;

typedef struct {
    bool has_r1;
    amount_t r1;
    bool has_r2;
    amount_t r2;
} reserves_t;

typedef struct {
    bool has_phi;
    trading_function_t phi;
    bool has_reserves;
    reserves_t reserves;
    bool close_on_fill;
} position_t;

typedef struct {
    bytes_t inner;
} position_id_t;

typedef struct {
    bool has_input;
    value_t input;
    bool has_output_id;
    asset_id_t output_id;
    bool has_max_output;
    amount_t max_output;
    bool has_min_output;
    amount_t min_output;
    uint64_t start_height;
    uint64_t end_height;
    uint64_t step_count;
} dutch_auction_description_t;

typedef struct {
    bytes_t inner;
} auction_id_t;

typedef struct {
    note_t note;
    uint64_t position;
    bytes_t randomizer;
    bytes_t value_blinding;
    uint8_t ui_address[SHORT_ADDRESS_LEN];
} spend_plan_t;

typedef struct {
    value_t value;
    address_plan_t dest_address;
    bytes_t rseed;
    bytes_t value_blinding;
    uint8_t ui_address[SHORT_ADDRESS_LEN];
} output_plan_t;

typedef struct {
    bool has_swap_plaintext;
    swap_plaintext_t swap_plaintext;
    bytes_t fee_blinding;
} swap_plan_t;

typedef struct {
    bool has_validator_identity;
    identity_key_t validator_identity;
    bool has_unbonded_amount;
    amount_t unbonded_amount;
} delegate_plan_t;

typedef struct {
    bool has_validator_identity;
    identity_key_t validator_identity;
    bool has_unbonded_amount;
    amount_t unbonded_amount;
    bool has_delegation_amount;
    amount_t delegation_amount;
    bool has_from_epoch;
    epoch_t from_epoch;
} undelegate_plan_t;

typedef struct {
    bool has_amount;
    amount_t amount;
    bool has_denom;
    denom_t denom;
    bytes_t destination_chain_address;
    bytes_t source_channel;
} ics20_withdrawal_plan_t;

typedef struct {
    bool has_validator_identity;
    identity_key_t validator_identity;
    bool has_penalty;
    penalty_t penalty;
    bool has_unbonding_amount;
    amount_t unbonding_amount;
    bytes_t balance_blinding;
    uint64_t unbonding_start_height;
} undelegate_claim_plan_t;

typedef struct {
    uint64_t proposal;
    uint64_t start_position;
    bool has_vote;
    uint8_t vote;
    bool has_staked_note;
    note_t staked_note;
    uint64_t staked_note_position;
    bool has_unbonded_amount;
    amount_t unbonded_amount;
    bytes_t randomizer;
} delegator_vote_plan_t;

typedef struct {
    bool has_position;
    position_t position;
} position_open_plan_t;

typedef struct {
    bool has_position_id;
    position_id_t position_id;
} position_close_plan_t;

typedef struct {
    bool has_reserves;
    reserves_t reserves;
    bool has_position_id;
    position_id_t position_id;
    bool has_pair;
    trading_pair_t pair;
    uint64_t sequence;
    value_t rewards[MAX_CALLBACK_ARRAY_SIZE];
    uint8_t rewards_qty;
} position_withdraw_plan_t;

typedef struct {
    bool has_description;
    dutch_auction_description_t description;
} action_dutch_auction_schedule_plan_t;

typedef struct {
    bool has_auction_id;
    auction_id_t auction_id;
} action_dutch_auction_end_plan_t;

typedef struct {
    bool has_auction_id;
    auction_id_t auction_id;
    uint64_t seq;
    bool has_reserves_input;
    value_t reserves_input;
    bool has_reserves_output;
    value_t reserves_output;
} action_dutch_auction_withdraw_plan_t;

typedef struct {
    address_plan_t return_address;
    bytes_t text;
} memo_plain_text_t;

typedef struct {
    memo_plain_text_t plaintext;
    bytes_t key;
    uint8_t ui_address[SHORT_ADDRESS_LEN];
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
        ics20_withdrawal_plan_t ics20_withdrawal;
        swap_plan_t swap;
        delegate_plan_t delegate;
        undelegate_plan_t undelegate;
        undelegate_claim_plan_t undelegate_claim;
        delegator_vote_plan_t delegator_vote;
        position_open_plan_t position_open;
        position_close_plan_t position_close;
        position_withdraw_plan_t position_withdraw;
        action_dutch_auction_schedule_plan_t action_dutch_auction_schedule;
        action_dutch_auction_end_plan_t action_dutch_auction_end;
        action_dutch_auction_withdraw_plan_t action_dutch_auction_withdraw;
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
    value_t fee;
    bytes_t data_bytes;
} parameters_t;

typedef struct {
    actions_hash_t actions;
    bool has_parameters;
    hash_t parameters_hash;
    bool has_memo;
    memo_plan_t memo;
    bool has_detection_data;
    detection_data_t detection_data;
} transaction_plan_t;

typedef struct {
    transaction_plan_t plan;
    action_t actions_plan[ACTIONS_QTY];
    parameters_t parameters_plan;
    uint8_t effect_hash[64];
} parser_tx_t;

typedef struct {
    uint8_t asset_id[ASSET_ID_LEN];
    const char symbol[MAX_SYMBOL_LEN];
    const char name[MAX_ASSET_NAME_LEN];
    uint16_t decimals;
} asset_info_t;

// This struct defines
// the metadata used to handle assets
// that are not listed in our internal table
// but that the user provide when signing a transaction
typedef struct {
    char denom[MAX_DENOM_LEN];
    uint8_t len;
} tx_metadata_t;

#ifdef __cplusplus
}
#endif
