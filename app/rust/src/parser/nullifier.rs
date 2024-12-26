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

use crate::keys::nk::NullifierKey;
use decaf377::Fq;
use poseidon377::hash_3;
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct Nullifier(pub Fq);

impl Nullifier {
    pub const LEN: usize = 32;
    pub const PROTO_LEN: usize = Self::LEN + 4;
    pub const PROTO_PREFIX: [u8; 4] = [0x32, 0x22, 0x0a, 0x20];

    /// Derive the [`Nullifier`] for a positioned note or swap given its [`merkle::Position`]
    /// and [`Commitment`].
    pub fn derive(nk: &NullifierKey, pos: u64, state_commitment: &Fq) -> Nullifier {
        Nullifier(hash_3(
            &Self::nullifier_domain_sep(),
            (nk.0, *state_commitment, (u64::from(pos)).into()),
        ))
    }

    fn nullifier_domain_sep() -> Fq {
        Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.nullifier").as_bytes())
    }

    pub fn to_proto(&self) -> [u8; Self::PROTO_LEN] {
        let mut proto = [0u8; Self::PROTO_LEN];
        proto[0..4].copy_from_slice(&Self::PROTO_PREFIX);
        proto[4..].copy_from_slice(&self.0.to_bytes());
        proto
    }
}
