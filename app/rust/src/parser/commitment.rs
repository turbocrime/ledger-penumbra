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

use decaf377::{Element, Fq};

#[derive(Clone, Debug)]
pub struct Commitment(pub Element);

#[derive(Clone, Debug)]
pub struct StateCommitment(pub Fq);

impl Commitment {
    pub const LEN: usize = 32; 
    pub const PROTO_LEN: usize = Self::LEN + 4;

    pub fn value_blinding_generator() -> decaf377::Element {
        let s = Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"decaf377-rdsa-binding").as_bytes());
        decaf377::Element::encode_to_curve(&s)
    }

    pub fn to_proto_spend(&self) -> [u8; Self::PROTO_LEN] {
        let mut proto = [0u8; Self::PROTO_LEN];
        proto[0..4].copy_from_slice(&[0x0a, 0x22, 0x0a, 0x20]);
        proto[4..].copy_from_slice(&self.0.vartime_compress().0);
        proto
    }

    pub fn to_proto_output(&self) -> [u8; Self::PROTO_LEN] {
        let mut proto = [0u8; Self::PROTO_LEN];
        proto[0..4].copy_from_slice(&[0x12, 0x22, 0x0a, 0x20]);
        proto[4..].copy_from_slice(&self.0.vartime_compress().0);
        proto
    }
}
