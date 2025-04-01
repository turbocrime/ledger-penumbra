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

use crate::constants::RK_LEN_BYTES;
use crate::protobuf_h::decaf377_rdsa_pb::{
    penumbra_crypto_decaf377_rdsa_v1_SpendVerificationKey_inner_tag, PB_LTYPE_UVARINT,
};
use crate::utils::protobuf::encode_proto_field;
use crate::ParserError;
use decaf377_rdsa::{SpendAuth, VerificationKey};

#[derive(Clone, PartialEq, Eq)]
pub struct Rk(pub VerificationKey<SpendAuth>);

impl Rk {
    pub const PROTO_LEN: usize = RK_LEN_BYTES + 2;

    pub fn to_proto(&self) -> Result<[u8; Self::PROTO_LEN], ParserError> {
        let mut proto = [0u8; Self::PROTO_LEN];

        let bytes = self.0.to_bytes();
        let len = encode_proto_field(
            penumbra_crypto_decaf377_rdsa_v1_SpendVerificationKey_inner_tag as u64,
            PB_LTYPE_UVARINT as u64,
            bytes.len(),
            &mut proto,
        )?;

        if len + bytes.len() != Self::PROTO_LEN {
            return Err(ParserError::InvalidLength);
        }

        proto[len..].copy_from_slice(&bytes);
        Ok(proto)
    }
}
