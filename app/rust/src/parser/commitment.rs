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

use crate::protobuf_h::asset_pb::{
    penumbra_core_asset_v1_BalanceCommitment_inner_tag, PB_LTYPE_UVARINT,
};
use crate::protobuf_h::tct_pb::penumbra_crypto_tct_v1_StateCommitment_inner_tag;
use crate::utils::protobuf::encode_proto_field;
use crate::ParserError;
use decaf377::{Element, Encoding, Fq};

#[derive(Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
// So instead of holding an Element
// this one stores an Encoding
// Element -> vartime_compress -> Encoding -> [u8; 32]
// [u8; 32] -> Encoding -> vartime_decompress -> Element -> Commitment
// so lets hold the compressed element to reduce
// binary size
pub struct Commitment(Encoding);
// pub struct Commitment(pub Element);

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct StateCommitment(pub Fq);

impl Commitment {
    pub const LEN: usize = 32;
    pub const PROTO_LEN: usize = Self::LEN + 2;

    pub fn value_blinding_generator() -> decaf377::Element {
        let s =
            Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"decaf377-rdsa-binding").as_bytes());
        decaf377::Element::encode_to_curve(&s)
    }

    pub fn to_proto(&self) -> Result<[u8; Self::PROTO_LEN], ParserError> {
        let mut proto = [0u8; Self::PROTO_LEN];

        let bytes = self.bytes_compress();
        let len = encode_proto_field(
            penumbra_core_asset_v1_BalanceCommitment_inner_tag as u64,
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

    /// Returns the vartime_compress byte representation
    /// of the internal defac377::Element
    pub fn bytes_compress(&self) -> [u8; Self::LEN] {
        self.0 .0
    }
}

impl From<Element> for Commitment {
    fn from(e: Element) -> Self {
        Commitment(e.vartime_compress())
    }
}

impl StateCommitment {
    pub const LEN: usize = 32;
    pub const PROTO_LEN: usize = Self::LEN + 2;

    pub fn to_proto(&self) -> Result<[u8; Self::PROTO_LEN], ParserError> {
        let mut proto = [0u8; Self::PROTO_LEN];

        let bytes = self.0.to_bytes();
        let len = encode_proto_field(
            penumbra_crypto_tct_v1_StateCommitment_inner_tag as u64,
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
