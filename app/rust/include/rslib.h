#pragma once

#include <stdint.h>

#include "coin.h"

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

// use to compute the full-viewing key
parser_error_t rs_compute_effect_hash();

parser_error_t rs_compute_transaction_plan(transaction_plan_t *plan, uint8_t *output, size_t output_len);

int32_t rs_bech32_encode(const uint8_t *hrp_ptr, size_t hrp_len, const uint8_t *data_ptr, size_t data_len,
                         uint8_t *output_ptr, size_t output_len);
