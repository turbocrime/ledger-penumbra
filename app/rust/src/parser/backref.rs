/*******************************************************************************
*   (c) 2024 Zondax AG
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

use crate::constants::NONCE_LEN;
use crate::parser::nullifier::Nullifier;
use crate::parser::symmetric::BackreferenceKey;
use crate::ParserError;
use chacha20poly1305::aead::{AeadInPlace, Nonce};
use chacha20poly1305::{ChaCha20Poly1305, KeyInit};
use decaf377::Fq;
pub const ENCRYPTED_BACKREF_LEN: usize = 48;

#[derive(Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct Backref {
    note_commitment: Fq,
}

#[derive(Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct EncryptedBackref {
    // We need to store the bytes as an array to avoid heap allocations
    // and be able to check if it's empty
    pub bytes: [u8; ENCRYPTED_BACKREF_LEN],
    // Track if the backref has actual data
    pub is_empty: bool,
}

impl Backref {
    pub fn new(note_commitment: Fq) -> Self {
        Self { note_commitment }
    }

    pub fn encrypt(
        &self,
        brk: &BackreferenceKey,
        nullifier: &Nullifier,
    ) -> Result<EncryptedBackref, ParserError> {
        let cipher = ChaCha20Poly1305::new(&brk.0);

        // Nonce is the first 12 bytes of the nullifier
        let nonce_bytes = &nullifier.0.to_bytes()[..NONCE_LEN];
        if nonce_bytes.len() < NONCE_LEN {
            return Err(ParserError::EncryptionError);
        }
        let nonce = Nonce::<ChaCha20Poly1305>::from_slice(nonce_bytes);
        let mut buffer = [0u8; ENCRYPTED_BACKREF_LEN];
        let commitment_bytes = self.note_commitment.to_bytes();
        let plaintext_len = commitment_bytes.len();

        // Copy commitment bytes to buffer
        buffer[..plaintext_len].copy_from_slice(&commitment_bytes);

        let tag = cipher
            .encrypt_in_place_detached(nonce, &[], &mut buffer[..plaintext_len])
            .map_err(|_| ParserError::EncryptionError)?;

        // Check if the buffer is too small to hold the encrypted data
        if plaintext_len + tag.len() > ENCRYPTED_BACKREF_LEN {
            return Err(ParserError::EncryptionError);
        }

        // Append authentication tag
        buffer[plaintext_len..plaintext_len + tag.len()].copy_from_slice(&tag);

        Ok(EncryptedBackref {
            bytes: buffer,
            is_empty: false,
        })
    }
}
