#pragma once

#include <stdint.h>

#include "coin.h"
#include "parser_common.h"

#ifdef __cplusplus
extern "C" {
#endif

void get_sr25519_sk(uint8_t *sk_ed25519_expanded);

void sign_sr25519_phase1(const uint8_t *sk_ed25519_expanded, const uint8_t *pk, const uint8_t *context_ptr,
                         uint32_t context_len, const uint8_t *msg_ptr, uint32_t msg_len, uint8_t *sig_ptr);
void sign_sr25519_phase2(const uint8_t *sk_ed25519_expanded, const uint8_t *pk, const uint8_t *context_ptr,
                         uint32_t context_len, const uint8_t *msg_ptr, uint32_t msg_len, uint8_t *sig_ptr);

void sign_decaf377(const uint8_t *msg, uint32_t msg_len, uint8_t *sig, uint32_t sig_len);

// To compute the raw-address associated to account and either it should be
// randomized or not(randomizer = NULL)
parser_error_t rs_compute_address(keys_t *keys, uint32_t account, uint8_t *randomizer);

// use to compute the full-viewing key
parser_error_t rs_compute_keys(keys_t *keys);
int32_t rs_bech32_encode(const uint8_t *hrp_ptr, size_t hrp_len, const uint8_t *data_ptr, size_t data_len,
                         uint8_t *output_ptr, size_t output_len);

parser_error_t rs_is_address_visible(const bytes_t *address, bool *is_visible, uint32_t *index);

parser_error_t rs_compute_effect_hash(transaction_plan_t *plan, uint8_t *output, size_t output_len);

parser_error_t rs_parameter_hash(bytes_t *data, uint8_t *output, size_t output_len);
parser_error_t rs_spend_action_hash(spend_plan_t *plan, uint8_t *output, size_t output_len);
parser_error_t rs_output_action_hash(output_plan_t *plan, bytes_t *memo_key, uint8_t *output, size_t output_len);
parser_error_t rs_swap_action_hash(swap_plan_t *plan, uint8_t *output, size_t output_len);
parser_error_t rs_undelegate_claim_action_hash(undelegate_claim_plan_t *plan, uint8_t *output, size_t output_len);
parser_error_t rs_delegator_vote_action_hash(delegator_vote_plan_t *plan, uint8_t *output, size_t output_len);
parser_error_t rs_position_withdraw_action_hash(position_withdraw_plan_t *plan, uint8_t *output, size_t output_len);
parser_error_t rs_action_dutch_auction_withdraw_action_hash(action_dutch_auction_withdraw_plan_t *plan, uint8_t *output,
                                                            size_t output_len);
parser_error_t rs_generic_action_hash(bytes_t *data, uint8_t action_type, uint8_t *output, size_t output_len);

parser_error_t rs_get_asset_id_from_metadata(const bytes_t *metadata, uint8_t *asset_id, uint16_t asset_id_len);

parser_error_t rs_sign_spend(const bytes_t *effect_hash, const bytes_t *randomizer, const spend_key_bytes_t *spend_key,
                             uint8_t *signature, uint16_t len);

#ifdef __cplusplus
}
#endif
