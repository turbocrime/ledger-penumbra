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

use crate::constants::SWAP_CIPHERTEXT_BYTES;
use crate::parser::commitment::Commitment;
use crate::parser::commitment::StateCommitment;
use crate::parser::swap_ciphertext::SwapCiphertext;
use crate::protobuf_h::dex_pb::{
    penumbra_core_component_dex_v1_SwapPayload_commitment_tag,
    penumbra_core_component_dex_v1_SwapPayload_encrypted_swap_tag, PB_LTYPE_UVARINT,
};
use crate::utils::protobuf::encode_proto_field;
use crate::ParserError;

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct SwapPayload {
    pub commitment: StateCommitment,
    pub encrypted_swap: SwapCiphertext,
}

impl SwapPayload {
    pub const PROTO_LEN: usize = SWAP_CIPHERTEXT_BYTES + Commitment::PROTO_LEN + 5;

    pub fn to_proto(&self) -> Result<[u8; Self::PROTO_LEN], ParserError> {
        let mut proto = [0u8; Self::PROTO_LEN];

        let mut offset = 0;
        let commitment = self.commitment.to_proto()?;
        offset += encode_proto_field(
            penumbra_core_component_dex_v1_SwapPayload_commitment_tag as u64,
            PB_LTYPE_UVARINT as u64,
            commitment.len(),
            &mut proto[offset..],
        )?;
        proto[offset..offset + commitment.len()].copy_from_slice(&commitment);
        offset += commitment.len();

        let encrypted_swap = self.encrypted_swap.0;
        offset += encode_proto_field(
            penumbra_core_component_dex_v1_SwapPayload_encrypted_swap_tag as u64,
            PB_LTYPE_UVARINT as u64,
            encrypted_swap.len(),
            &mut proto[offset..],
        )?;
        proto[offset..offset + encrypted_swap.len()].copy_from_slice(&encrypted_swap);
        offset += encrypted_swap.len();

        if offset != Self::PROTO_LEN {
            return Err(ParserError::InvalidLength);
        }

        Ok(proto)
    }
}
