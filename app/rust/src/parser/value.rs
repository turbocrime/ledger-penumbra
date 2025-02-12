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

use crate::constants::{AMOUNT_LEN_BYTES, ID_LEN_BYTES};
use crate::parser::bytes::BytesC;
use crate::parser::{
    amount::{Amount, AmountC},
    id::{Id, IdC},
    ParserError,
};
use crate::protobuf_h::asset_pb::{
    penumbra_core_asset_v1_Value_amount_tag, penumbra_core_asset_v1_Value_asset_id_tag,
    PB_LTYPE_UVARINT,
};
use crate::utils::protobuf::{encode_proto_field, encode_varint};

#[derive(Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub enum Sign {
    Required,
    Provided,
}

impl PartialEq for Sign {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Sign::Required, Sign::Required) | (Sign::Provided, Sign::Provided)
        )
    }
}

#[derive(Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct Value {
    pub amount: Amount,
    // The asset ID. 256 bits.
    pub asset_id: Id,
}

#[derive(Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct Imbalance {
    pub value: Value,
    pub sign: Sign,
}

impl Value {
    pub const LEN: usize = AMOUNT_LEN_BYTES + ID_LEN_BYTES;
    pub const PROTO_LEN: usize = Amount::PROTO_LEN + Id::PROTO_LEN + 1;

    pub fn to_bytes(&self) -> Result<[u8; Self::LEN], ParserError> {
        let mut bytes = [0; Self::LEN];
        bytes[0..AMOUNT_LEN_BYTES].copy_from_slice(&self.amount.to_le_bytes());
        bytes[AMOUNT_LEN_BYTES..AMOUNT_LEN_BYTES + ID_LEN_BYTES]
            .copy_from_slice(&self.asset_id.to_bytes());
        Ok(bytes)
    }

    pub fn to_proto(&self) -> Result<([u8; Self::PROTO_LEN], usize), ParserError> {
        let (value_amount, value_amount_len) = self.amount.to_proto()?;
        let mut proto = [0u8; Self::PROTO_LEN];
        let mut offset = 0;

        // Encode the amount
        let amount_tag = (penumbra_core_asset_v1_Value_amount_tag << 3 | PB_LTYPE_UVARINT) as u64;
        let mut tag_buf = [0u8; 10];
        offset += encode_varint(amount_tag, &mut tag_buf)?;
        if offset + value_amount_len > proto.len() {
            return Err(ParserError::InvalidLength);
        }
        proto[..offset].copy_from_slice(&tag_buf[..offset]);
        proto[offset..offset + value_amount_len].copy_from_slice(&value_amount[..value_amount_len]);
        offset += value_amount_len;

        // Encode the asset ID into the proto buffer
        let asset_id_proto = self.asset_id.to_proto()?;
        offset += encode_proto_field(
            penumbra_core_asset_v1_Value_asset_id_tag as u64,
            PB_LTYPE_UVARINT as u64,
            asset_id_proto.len(),
            &mut proto[offset..],
        )?;

        if offset + Id::PROTO_LEN > proto.len() {
            return Err(ParserError::InvalidLength);
        }
        proto[offset..offset + Id::PROTO_LEN].copy_from_slice(&asset_id_proto);

        Ok((proto, offset + Id::PROTO_LEN))
    }
}

#[repr(C)]
#[derive(Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct ValueC {
    pub has_amount: bool,
    pub amount: AmountC,
    pub has_asset_id: bool,
    pub asset_id: IdC,
}

impl TryFrom<ValueC> for Value {
    type Error = ParserError;

    fn try_from(value: ValueC) -> Result<Self, Self::Error> {
        Ok(Value {
            amount: value.amount.try_into()?,
            asset_id: value.asset_id.try_into()?,
        })
    }
}

impl Default for ValueC {
    fn default() -> Self {
        let amount = AmountC { lo: 0, hi: 0 };
        let asset_id = IdC {
            inner: BytesC::default(),
        };
        ValueC {
            has_amount: false,
            amount,
            has_asset_id: false,
            asset_id,
        }
    }
}
