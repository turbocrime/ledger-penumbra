/*******************************************************************************
*   (c) 2024 Zondax GmbH
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

use crate::constants::{NONCE_LEN, NONCE_MEMO, NONCE_MEMO_KEYS, NONCE_NOTE, NONCE_SWAP};
use crate::ParserError;
use chacha20poly1305::aead::{AeadInPlace, Nonce};
use chacha20poly1305::{ChaCha20Poly1305, Key, KeyInit};
use crate::keys::ovk::Ovk;
use decaf377::Fq;
use crate::keys::ka;
use crate::parser::commitment::{Commitment, StateCommitment};
use rand::{CryptoRng, RngCore};

pub const PAYLOAD_KEY_LEN_BYTES: usize = 32;
pub const OVK_WRAPPED_LEN_BYTES: usize = 48;
pub const MEMOKEY_WRAPPED_LEN_BYTES: usize = 48;

/// Represents the item to be encrypted/decrypted with the [`PayloadKey`].
#[derive(Copy, Clone, PartialEq, Eq)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub enum PayloadKind {
    /// Note is action-scoped.
    Note,
    /// MemoKey is action-scoped.
    MemoKey,
    /// Memo is transaction-scoped.
    Memo,
    /// Swap is action-scoped.
    Swap,
}

impl PayloadKind {
    pub(crate) fn nonce(&self) -> [u8; NONCE_LEN] {
        match self {
            Self::Note => *NONCE_NOTE,
            Self::MemoKey => *NONCE_MEMO_KEYS,
            Self::Swap => *NONCE_SWAP,
            Self::Memo => *NONCE_MEMO,
        }
    }
}

/// Represents a symmetric `ChaCha20Poly1305` key.
///
/// Used for encrypting and decrypting notes, swaps, memos, and memo keys.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PayloadKey(Key);

impl PayloadKey {
        /// Use Blake2b-256 to derive a `PayloadKey`.
        pub fn derive(shared_secret: &ka::SharedSecret, epk: &ka::Public) -> Self {
            let mut kdf_params = blake2b_simd::Params::new();
            kdf_params.personal(b"Penumbra_Payload");
            kdf_params.hash_length(32);
            let mut kdf = kdf_params.to_state();
            kdf.update(&shared_secret.0);
            kdf.update(&epk.0);
    
            let key = kdf.finalize();
            Self(*Key::from_slice(key.as_bytes()))
        }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), PAYLOAD_KEY_LEN_BYTES, "Invalid key length");
        Self(*Key::from_slice(bytes))
    }

    /// Derive a random `PayloadKey`. Used for memo key wrapping.
    pub fn random_key<R: CryptoRng + RngCore>(rng: &mut R) -> Self {
        let mut key_bytes = [0u8; 32];
        rng.fill_bytes(&mut key_bytes);
        Self(*Key::from_slice(&key_bytes[..]))
    }

    // Encrypt a note, memo, or memo key using the `PayloadKey`.
    pub fn encrypt(&self, plaintext: &mut [u8], kind: PayloadKind, text_len: usize) -> Result<(), ParserError> {
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&self.0));
        let nonce_bytes = kind.nonce();
        let nonce = Nonce::<ChaCha20Poly1305>::from_slice(&nonce_bytes);

        let plaintext_len = plaintext.len();
        let tag = cipher
            .encrypt_in_place_detached(nonce, b"", &mut plaintext[..text_len])
            .map_err(|_| ParserError::UnexpectedError)?;

        plaintext[plaintext_len - 16..].copy_from_slice(&tag);

        Ok(())
    }

    /// Use Blake2b-256 to derive an encryption key from the OVK and public fields for swaps.
    pub fn derive_swap(ovk: &Ovk, cm: StateCommitment) -> Self {
        let cm_bytes: [u8; 32] = cm.0.to_bytes();
    
        let mut kdf_params = blake2b_simd::Params::new();
        kdf_params.personal(b"Penumbra_Payswap");
        kdf_params.hash_length(32);
        let mut kdf = kdf_params.to_state();
        kdf.update(&ovk.to_bytes());
        kdf.update(&cm_bytes);
    
        let key = kdf.finalize();
        Self(*Key::from_slice(key.as_bytes()))
    }

    /// Encrypt a swap using the `PayloadKey`.
    pub fn encrypt_swap(&self, plaintext: &mut [u8], text_len: usize) -> Result<(), ParserError> {
        let cipher = ChaCha20Poly1305::new(&self.0);
        let nonce_bytes = PayloadKind::Swap.nonce();
        let nonce = Nonce::<ChaCha20Poly1305>::from_slice(&nonce_bytes);

        let plaintext_len = plaintext.len();

        let tag = cipher
            .encrypt_in_place_detached(nonce, b"", &mut plaintext[..text_len])
            .map_err(|_| ParserError::UnexpectedError)?;

            plaintext[plaintext_len - 16..].copy_from_slice(&tag);

        Ok(())
    }
}


/// Represents encrypted key material used to reconstruct a `PayloadKey`.
#[derive(Clone, Debug)]
pub struct OvkWrappedKey(pub [u8; OVK_WRAPPED_LEN_BYTES]);

impl OvkWrappedKey {
    pub const PROTO_LEN: usize = OVK_WRAPPED_LEN_BYTES + 2;
    pub const PROTO_PREFIX: [u8; 2] = [0x22, 0x30];

    pub fn to_proto(&self) -> [u8; Self::PROTO_LEN] {
        let mut proto = [0u8; Self::PROTO_LEN];
        proto[0..2].copy_from_slice(&Self::PROTO_PREFIX);
        proto[2..].copy_from_slice(&self.0);
        proto
    }
}

/// Represents a symmetric `ChaCha20Poly1305` key.
///
/// Used for encrypting and decrypting [`OvkWrappedKey`] material used to decrypt
/// outgoing notes, and memos.
pub struct OutgoingCipherKey(Key);
impl OutgoingCipherKey {
    pub const PROTO_LEN: usize = OVK_WRAPPED_LEN_BYTES + 4;

    /// Use Blake2b-256 to derive an encryption key `ock` from the OVK and public fields.
    pub fn derive(
        ovk: &Ovk,
        cv: Commitment,
        cm: &Fq,
        epk: &ka::Public,
    ) -> Self {
        let cv_bytes: [u8; 32] = cv.0.vartime_compress().0;
        let cm_bytes: [u8; 32] = cm.to_bytes();

        let mut kdf_params = blake2b_simd::Params::new();
        kdf_params.hash_length(32);
        kdf_params.personal(b"Penumbra_OutCiph");
        let mut kdf = kdf_params.to_state();
        kdf.update(&ovk.to_bytes());
        kdf.update(&cv_bytes);
        kdf.update(&cm_bytes);
        kdf.update(&epk.0);

        let key = kdf.finalize();
        Self(*Key::from_slice(key.as_bytes()))
    }

    /// Encrypt key material using the `OutgoingCipherKey`.
    pub fn encrypt(&self, plaintext: &mut [u8], kind: PayloadKind) -> Result<(), ParserError> {
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&self.0));

        // Note: Here we use the same nonce as note encryption, however the keys are different.
        // For note encryption we derive the `PayloadKey` symmetric key from the shared secret and epk.
        // However, for the outgoing cipher key, we derive a symmetric key from the
        // sender's OVK, balance commitment, note commitment, and the epk. Since the keys are
        // different, it is safe to use the same nonce.
        //
        // References:
        // * Section 5.4.3 of the ZCash protocol spec
        // * Section 2.3 RFC 7539
        let nonce_bytes = kind.nonce();
        let nonce = Nonce::<ChaCha20Poly1305>::from_slice(&nonce_bytes);


        let plaintext_len = plaintext.len();

        let tag = cipher
            .encrypt_in_place_detached(nonce, b"", &mut plaintext[..32])
            .map_err(|_| ParserError::UnexpectedError)?;
        plaintext[plaintext_len - 16..].copy_from_slice(&tag);
        Ok(())
    }
}


#[derive(Clone, Debug)]
pub struct WrappedMemoKey(pub [u8; MEMOKEY_WRAPPED_LEN_BYTES]);

impl WrappedMemoKey {
    pub const PROTO_LEN: usize = MEMOKEY_WRAPPED_LEN_BYTES + 2;
    pub const PROTO_PREFIX: [u8; 2] = [0x1a, 0x30];

    /// Encrypt a memo key using the action-specific `PayloadKey`.
    pub fn encrypt(
        memo_key: &PayloadKey,
        esk: ka::Secret,
        transmission_key: &ka::Public,
        diversified_generator: &decaf377::Element,
    ) -> Result<Self, ParserError> {
        // 1. Construct the per-action PayloadKey.
        let epk = esk.diversified_public(diversified_generator);
        let shared_secret = esk
            .key_agreement_with(transmission_key)
            .map_err(|_| ParserError::UnexpectedError)?;

        let action_key = PayloadKey::derive(&shared_secret, &epk);

        // 2. Now use the per-action key to encrypt the memo key.
        let mut encryption_result = [0u8; OVK_WRAPPED_LEN_BYTES];
        encryption_result[..memo_key.0.len()].copy_from_slice(&memo_key.0);

        action_key.encrypt(&mut encryption_result, PayloadKind::MemoKey, 32).map_err(|_| ParserError::UnexpectedError).unwrap();
        let wrapped_memo_key_bytes: [u8; MEMOKEY_WRAPPED_LEN_BYTES] = encryption_result
            .try_into()
            .map_err(|_| ParserError::UnexpectedError)?;

        Ok(WrappedMemoKey(wrapped_memo_key_bytes))
    }

    pub fn to_proto(&self) -> [u8; Self::PROTO_LEN] {
        let mut proto = [0u8; Self::PROTO_LEN];
        proto[0..2].copy_from_slice(&Self::PROTO_PREFIX);
        proto[2..].copy_from_slice(&self.0);
        proto
    }
}