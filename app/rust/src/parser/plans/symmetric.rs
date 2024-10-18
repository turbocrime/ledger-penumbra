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
    pub fn encrypt(&self, plaintext: &mut [u8], kind: PayloadKind) -> Result<(), ParserError> {
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&self.0));
        let nonce_bytes = kind.nonce();
        let nonce = Nonce::<ChaCha20Poly1305>::from_slice(&nonce_bytes);

        let plaintext_len = plaintext.len();
        let tag = cipher
            .encrypt_in_place_detached(nonce, b"", &mut plaintext[..512])
            .map_err(|_| ParserError::UnexpectedError)?;

        plaintext[plaintext_len - 16..].copy_from_slice(&tag);

        Ok(())
    }
}

