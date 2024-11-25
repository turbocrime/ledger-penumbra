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

/// A hash of a transaction's _effecting data_, describing its effects on the
/// chain state.
///
/// This includes, e.g., the commitments to new output notes created by the
/// transaction, or nullifiers spent by the transaction, but does not include
/// _authorizing data_ such as signatures or zk proofs.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct EffectHash(pub [u8; 64]);

/// A helper function to create a BLAKE2b `State` instance given a variable-length personalization string.
pub fn create_personalized_state(personalization: &str) -> blake2b_simd::State {
    let mut state = blake2b_simd::State::new();

    // The `TypeUrl` provided as a personalization string is variable length,
    // so we first include the length in bytes as a fixed-length prefix.
    let length = personalization.len() as u64;
    state.update(&length.to_le_bytes());
    state.update(personalization.as_bytes());

    state
}

impl EffectHash {
    pub fn from_array(array: [u8; 64]) -> Self {
        EffectHash(array)
    }

    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }

    pub fn from_proto_effecting_data(personalization: &str, data: &[u8]) -> EffectHash {
        let mut state = create_personalized_state(personalization);
        state.update(data);

        EffectHash(*state.finalize().as_array())
    }
}

impl AsRef<[u8]> for EffectHash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Default for EffectHash {
    fn default() -> Self {
        Self([0u8; 64])
    }
}

impl EffectHash {
    pub fn as_array(&self) -> &[u8; 64] {
        &self.0
    }
}

#[cfg(test)]
impl std::fmt::Debug for EffectHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("EffectHash")
            .field(&hex::encode(self.0))
            .finish()
    }
}
