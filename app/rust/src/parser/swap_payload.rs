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

use crate::parser::commitment::StateCommitment;
use crate::parser::swap_ciphertext::SwapCiphertext;
use crate::parser::commitment::Commitment;

#[derive(Clone, Debug)]
pub struct SwapPayload {
    pub commitment: StateCommitment,
    pub encrypted_swap: SwapCiphertext,
}

impl SwapPayload {
    pub const PROTO_LEN: usize = SwapCiphertext::PROTO_LEN + Commitment::PROTO_LEN + 3;
    pub const PROTO_PREFIX: [u8; 3] = [0x2a, 0xb7, 0x02];

    pub fn to_proto(&self) -> [u8; Self::PROTO_LEN] {
        let mut proto = [0u8; Self::PROTO_LEN];

        proto[0..3].copy_from_slice(&Self::PROTO_PREFIX); 
        proto[3..Commitment::PROTO_LEN + 3].copy_from_slice(&self.commitment.to_proto_swap());
        proto[Commitment::PROTO_LEN + 3..Self::PROTO_LEN].copy_from_slice(&self.encrypted_swap.to_proto());

        proto
    }
}
