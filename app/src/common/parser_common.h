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

#include "constants.h"
#include "keys_def.h"
#include "parser_txdef.h"

#define CHECK_ERROR(__CALL)                   \
    {                                         \
        parser_error_t __err = __CALL;        \
        CHECK_APP_CANARY()                    \
        if (__err != parser_ok) return __err; \
    }

// Convert bytes to uint32_t,
// assume data is in BE format
#define U32_BE(buffer, number)                                                                         \
    do {                                                                                               \
        (number) = (uint32_t)((buffer)[0] << 24 | (buffer)[1] << 16 | (buffer)[2] << 8 | (buffer)[3]); \
    } while (0)

typedef enum {
    // Success
    parser_ok = 0,

    // Generic errors
    parser_no_data,
    parser_init_context_empty,
    parser_display_idx_out_of_range,
    parser_display_page_out_of_range,
    parser_unexpected_error,

    // Method/Version related
    parser_unexpected_method,
    parser_unexpected_version,
    parser_unexpected_characters,

    // Field related
    parser_duplicated_field,
    parser_missing_field,
    parser_unexpected_field,

    // Transaction related
    parser_unknown_transaction,
    parser_invalid_transaction_type,

    // Plan related
    parser_spend_plan_error,
    parser_output_plan_error,
    parser_delegate_plan_error,
    parser_undelegate_plan_error,
    parser_ics20_withdrawal_plan_error,
    parser_swap_plan_error,
    parser_parameter_hash_error,
    parser_effect_hash_error,
    parser_undelegate_claim_plan_error,
    parser_delegator_vote_plan_error,
    parser_position_withdraw_plan_error,
    parser_action_dutch_auction_schedule_plan_error,
    parser_action_dutch_auction_end_plan_error,
    parser_action_dutch_auction_withdraw_plan_error,

    // Chain related
    parser_invalid_chain_id,
    parser_unexpected_chain,

    // Cryptographic and key-related errors
    parser_invalid_hash_mode,
    parser_invalid_signature,
    parser_invalid_pubkey_encoding,
    parser_invalid_address_version,
    parser_invalid_address_length,
    parser_invalid_type_id,
    parser_invalid_codec,
    parser_invalid_threshold,
    parser_invalid_network_id,
    parser_invalid_ascii_value,
    parser_invalid_timestamp,
    parser_invalid_staking_amount,
    parser_unexpected_type,
    parser_operation_overflows,
    parser_unexpected_buffer_end,
    parser_unexpected_number_items,
    parser_value_out_of_range,
    parser_invalid_address,
    parser_invalid_path,
    parser_invalid_length,
    parser_too_many_outputs,
    parser_unexpected_data,
    parser_invalid_clue_key,
    parser_invalid_tx_key,
    parser_invalid_fq,
    parser_invalid_detection_key,
    parser_invalid_fvk,
    parser_invalid_ivk,
    parser_invalid_key_len,
    parser_invalid_action_type,
    parser_invalid_precision,
    parser_precision_too_large,
    parser_clue_creation_failed,
    parser_invalid_asset_id,
    parser_detection_data_overflow,
    parser_actions_overflow,
    parser_invalid_metadata,
    parser_invalid_signature_len,
    parser_overflow,
    parser_non_integral,
    parser_unexpected_value,
} parser_error_t;

typedef struct {
    const uint8_t *buffer;
    uint16_t bufferLen;
    uint16_t offset;
    parser_tx_t *tx_obj;
    address_index_t address_index;
    tx_metadata_t tx_metadata[MAX_TX_METADATA_LEN];
    uint8_t tx_metadata_len;
} parser_context_t;

#ifdef __cplusplus
}
#endif
