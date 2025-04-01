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

use crate::constants::ID_LEN_BYTES;
use crate::parser::bytes::BytesC;
use crate::protobuf_h::asset_pb::{penumbra_core_asset_v1_AssetId_inner_tag, PB_LTYPE_UVARINT};
use crate::utils::prf::expand_fq;
use crate::utils::protobuf::encode_proto_field;
use crate::ParserError;
use decaf377::Fq;

#[derive(Clone, Default)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct Id(pub Fq);

impl Id {
    pub const LEN: usize = ID_LEN_BYTES;
    pub const PROTO_LEN: usize = Self::LEN + 2;

    /// Compute the value generator   for this asset, used for computing balance commitments.
    pub fn value_generator(&self) -> decaf377::Element {
        decaf377::Element::encode_to_curve(&poseidon377::hash_1(
            &Self::value_generator_domain_sep(),
            self.0,
        ))
    }

    pub fn value_generator_domain_sep() -> Fq {
        Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.value.generator").as_bytes())
    }

    pub fn to_bytes(&self) -> [u8; Self::LEN] {
        let mut bytes = [0; Self::LEN];
        bytes.copy_from_slice(&self.0.to_bytes());
        bytes
    }

    pub fn to_proto(&self) -> Result<[u8; Self::PROTO_LEN], ParserError> {
        let mut proto = [0u8; Self::PROTO_LEN];

        let bytes = self.to_bytes();
        let len = encode_proto_field(
            penumbra_core_asset_v1_AssetId_inner_tag as u64,
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

impl PartialEq for Id {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[repr(C)]
#[derive(Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct IdC {
    pub inner: BytesC,
}

impl IdC {
    pub fn get_inner(&self) -> Result<&[u8], ParserError> {
        self.inner.get_bytes()
    }
}

impl TryFrom<IdC> for Id {
    type Error = ParserError;

    fn try_from(value: IdC) -> Result<Self, Self::Error> {
        let inner = value.get_inner()?;
        Ok(Id(Fq::from_le_bytes_mod_order(inner)))
    }
}

#[derive(Clone, Default)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct AssetId(Id);

impl AssetId {
    const ASSET_ID_PERSONAL: &'static [u8; 16] = b"Penumbra_AssetID";

    pub fn new(denom: &str) -> Result<Self, ParserError> {
        let fq = expand_fq::expand_ff(Self::ASSET_ID_PERSONAL, &[], denom.as_bytes())?;
        Ok(Self(Id(fq)))
    }

    pub fn to_bytes(&self) -> [u8; ID_LEN_BYTES] {
        self.0.to_bytes()
    }
}

#[derive(Clone, Default)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct IdRaw(pub [u8; 32]);

impl IdRaw {
    pub const PROTO_LEN: usize = ID_LEN_BYTES + 2;

    pub fn to_proto(&self) -> Result<[u8; Self::PROTO_LEN], ParserError> {
        let mut proto = [0u8; Self::PROTO_LEN];

        let bytes = self.0;
        let len = encode_proto_field(
            penumbra_core_asset_v1_AssetId_inner_tag as u64,
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
