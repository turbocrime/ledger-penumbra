/*******************************************************************************
 *  (c) 2018 - 2024 Zondax AG
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

#define KEY_LEN 32
#define FVK_LEN 64
#define DIVERSIFIER_KEY_LEN 16
#define OUTGOING_VIEWING_KEY_LEN KEY_LEN
#define NULLIFIER_KEY_LEN KEY_LEN            // Assuming decaf377 curve parameters
#define SPEND_VERIFICATION_KEY_LEN KEY_LEN   // Assuming encoded size
#define SPEND_AUTHORIZATION_KEY_LEN KEY_LEN  // Assuming encoded size
#define INCOMING_VIEWING_KEY_LEN KEY_LEN     // Assuming modulo r size
#define ADDR_MAX_ENC_LEN 150                 // The maximun length of the encoded address

#define SIGNATURE_LEN 64

/// Number of bits in the address short form divided by the number of bits per Bech32m character
#define ADDRESS_NUM_CHARS_SHORT_FORM 24

#define DIVERSIFIER_KEY_LEN 16

#define ADDR_RANDOMIZER_LEN 12

#define ADDRESS_NUM_CHARS_SHORT_FORM 24
#define NUM_CHARS_TO_DISPLAY 33

// raw keys in bytes

/** The spending key consists of spend_key_bytes and ask.
(Since ask is derived from spend_key_bytes, only the spend_key_bytes need to be stored,
 but the ask is considered part of the spending key).**/
typedef uint8_t spend_key_bytes_t[KEY_LEN];

// ask = from_le_bytes(prf_expand("Penumbra_ExpndSd", spend_key_bytes, 0)) mod r
// nk  = from_le_bytes(prf_expand("Penumbra_ExpndSd", spend_key_bytes, 0)) mod q

// spend authorization key that is derived from spend_key_bytes
typedef uint8_t ask_t[SPEND_AUTHORIZATION_KEY_LEN];
// nullifier key that is derived from spend_key_bytes
typedef uint8_t nk_t[NULLIFIER_KEY_LEN];

// spend verification key that is derived from ask
typedef uint8_t ak_t[SPEND_VERIFICATION_KEY_LEN];

// The full viewing key consists of two components:
//
//     $\mathsf{ak} \in \mathbb G$, the spend verification key, a decaf377-rdsa verification key;
//     pub struct FqMontgomeryDomainFieldElement(pub [u32; 8]);

//     $\mathsf{nk} \in \mathbb F_q$, the nullifier key.
//
typedef uint8_t full_viewing_key_t[KEY_LEN * 2];

// ovk  = prf_expand(b"Penumbra_DeriOVK", to_le_bytes(nk), decaf377_encode(ak))[0..32]
// ivk = poseidon_hash_2(from_le_bytes(b"penumbra.derive.ivk"), nk, decaf377_s(ak)) mod r
// dk = prf_expand(b"Penumbra_DerivDK", to_le_bytes(nk), decaf377_encode(ak))[0..16]
typedef uint8_t ovk_t[OUTGOING_VIEWING_KEY_LEN];
typedef uint8_t ivk_t[INCOMING_VIEWING_KEY_LEN];

// computed using ak and nf keys
// diversifier_key is required for computing a diversifier that along with the tag
// can be used to derive a payment address.
typedef uint8_t diversifier_key_t[DIVERSIFIER_KEY_LEN];

// 16-byte tags used to derive up to $2^{128}$ distinct addresses for each spending authority
typedef uint8_t diversifier_tag_t[16];

// A bech32m encoded address
typedef uint8_t address_t[ADDRESS_LEN_BYTES];
// A signature type
typedef uint8_t signature_t[SIGNATURE_LEN];

typedef struct {
    spend_key_bytes_t skb;
    full_viewing_key_t fvk;
    address_t address;
} keys_t;

typedef struct {
    uint32_t account;
    uint8_t has_randomizer;
    uint8_t randomizer[ADDR_RANDOMIZER_LEN];
} __attribute__((packed)) address_index_t;

#ifdef __cplusplus
}
#endif
