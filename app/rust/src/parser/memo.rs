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

use super::memo_plain_text::MemoPlaintextC;
use super::symmetric::{PayloadKey, PayloadKind};
use crate::constants::{MEMO_CIPHERTEXT_LEN_BYTES, MEMO_LEN_BYTES};
use crate::ParserError;
use crate::parser::effect_hash::{create_personalized_state, EffectHash};
use crate::parser::bytes::BytesC;
use crate::utils::protobuf::encode_varint;

#[repr(C)]
#[derive(Default)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct MemoPlanC {
    pub plaintext: MemoPlaintextC,
    pub key: BytesC,
}

impl MemoPlanC {
    pub fn effect_hash(&self) -> Result<EffectHash, ParserError> {
        if self.is_empty() {
            return Ok(EffectHash::default());
        }

        MemoCiphertext::encrypt(&self.key, &self.plaintext)
            .map(|ciphertext| ciphertext.effect_hash().finalize())
            .map(|hash| EffectHash::from_array(*hash.as_array()))
    }

    pub fn is_empty(&self) -> bool {
        self.plaintext.return_address.inner.len == 0
            && self.plaintext.return_address.alt_bech32m.len == 0
            && self.plaintext.text.len == 0
            && self.key.len == 0
    }

    pub fn get_memo_key(&self) -> Result<&[u8], ParserError> {
        self.key.get_bytes()
    }
}

pub struct MemoCiphertext(pub [u8; MEMO_CIPHERTEXT_LEN_BYTES]);

impl MemoCiphertext {
    /// Encrypt a memo, returning its ciphertext.
    pub fn encrypt(memo_key: &BytesC, memo: &MemoPlaintextC) -> Result<Self, ParserError> {
        let mut ciphertext = [0u8; MEMO_CIPHERTEXT_LEN_BYTES];

        let return_address_bytes = memo.return_address.inner.get_bytes()?;
        let text_bytes = match memo.text.get_bytes() {
            Ok(bytes) => bytes,
            Err(_) => &[],
        };

        if (memo.return_address.inner.len as usize + memo.text.len as usize) > MEMO_LEN_BYTES {
            return Err(ParserError::InvalidLength);
        }

        // Copy return_address_bytes to ciphertext buffer
        ciphertext[..return_address_bytes.len()].copy_from_slice(return_address_bytes);
        // Copy text_bytes to ciphertext buffer after return_address_bytes
        ciphertext[return_address_bytes.len()..(return_address_bytes.len() + text_bytes.len())]
            .copy_from_slice(text_bytes);

        // Get memo key bytes
        let memo_key_bytes = memo_key.get_bytes()?;

        // Create PayloadKey and encrypt
        let key = PayloadKey::from_bytes(memo_key_bytes);
        key.encrypt(&mut ciphertext, PayloadKind::Memo, MEMO_LEN_BYTES)
            .map_err(|_| ParserError::UnexpectedError)?;


        Ok(MemoCiphertext(ciphertext))
    }

    pub fn effect_hash(&self) -> blake2b_simd::State {
        let mut state = create_personalized_state("/penumbra.core.transaction.v1.MemoCiphertext");

        // Encode the length of bytes like protobuf encoding with tag = 1
        let len = MEMO_CIPHERTEXT_LEN_BYTES;
        // Max size needed for u64 varint + 1 byte tag
        let mut tag_and_len = [0u8; 11];
        tag_and_len[0] = 0x0A; // Tag
        let varint_len = encode_varint(len as u64, &mut tag_and_len[1..]);

        state.update(&tag_and_len[..varint_len + 1]);
        state.update(&self.0);
        state
    }
}
