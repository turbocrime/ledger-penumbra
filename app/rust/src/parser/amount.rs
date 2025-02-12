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

use crate::constants::AMOUNT_LEN_BYTES;
use crate::parser::fixpoint::U128x128;
use crate::protobuf_h::num_pb::{
    penumbra_core_num_v1_Amount_hi_tag, penumbra_core_num_v1_Amount_lo_tag,
};
use crate::utils::protobuf::encode_varint;
use crate::ParserError;
use decaf377::{Fq, Fr};
use std::ops;

#[derive(Copy, Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct Amount {
    pub inner: u128,
}

impl Amount {
    pub const LEN: usize = AMOUNT_LEN_BYTES;
    pub const PROTO_LEN: usize = 23; // 2*10 to encode the lo and hi, plus 2 for each tag plus 1 for the size
    pub const PROTO_PREFIX_LO: [u8; 1] = [(penumbra_core_num_v1_Amount_lo_tag << 3) as u8]; // (1 << 3) | 0 = 8
    pub const PROTO_PREFIX_HI: [u8; 1] = [(penumbra_core_num_v1_Amount_hi_tag << 3) as u8]; // (2 << 3) | 0 = 16

    pub fn to_le_bytes(&self) -> [u8; Self::LEN] {
        self.inner.to_le_bytes()
    }

    pub fn to_proto(&self) -> Result<([u8; Self::PROTO_LEN], usize), ParserError> {
        let mut output_buffer = [0u8; Self::PROTO_LEN];
        let mut temp_buffer = [0u8; Self::PROTO_LEN];
        let mut pos = 0;

        // Split u128 into high and low u64 parts
        let lo = self.inner as u64;
        let hi = (self.inner >> 64) as u64;

        // Encode low part if non-zero
        if lo != 0 {
            temp_buffer[pos] = Self::PROTO_PREFIX_LO[0];
            pos += 1;
            pos += encode_varint(lo, &mut temp_buffer[pos..])?;
        }

        // Encode high part if non-zero
        if hi != 0 {
            temp_buffer[pos] = Self::PROTO_PREFIX_HI[0];
            pos += 1;
            pos += encode_varint(hi, &mut temp_buffer[pos..])?;
        }

        // Encode the length prefix
        let prefix_len = encode_varint(pos as u64, &mut output_buffer)?;

        // Check if the total length exceeds our buffer capacity
        let total_length = prefix_len + pos;
        if total_length > Self::PROTO_LEN {
            return Err(ParserError::InvalidLength);
        }

        // Copy the encoded data to the output buffer
        output_buffer[prefix_len..total_length].copy_from_slice(&temp_buffer[..pos]);

        Ok((output_buffer, total_length))
    }
}

impl TryFrom<AmountC> for Amount {
    type Error = ParserError;

    fn try_from(amount: AmountC) -> Result<Self, Self::Error> {
        let lo = amount.lo as u128;
        let hi = amount.hi as u128;
        // `hi` and `lo` represent the high/low order bytes respectively.
        //
        // We want to decode `hi` and `lo` into a single `u128` of the form:
        //
        //            hi: u64                          lo: u64
        // ┌───┬───┬───┬───┬───┬───┬───┬───┐ ┌───┬───┬───┬───┬───┬───┬───┬───┐
        // │   │   │   │   │   │   │   │   │ │   │   │   │   │   │   │   │   │
        // └───┴───┴───┴───┴───┴───┴───┴───┘ └───┴───┴───┴───┴───┴───┴───┴───┘
        //   15  14  13  12  11  10  9   8     7   6   5   4   3   2   1   0
        //
        // To achieve this, we shift `hi` 8 bytes to the left:
        let shifted = hi << 64;
        // and then add the lower order bytes:
        let inner = shifted + lo;

        Ok(Amount { inner })
    }
}

impl From<Amount> for U128x128 {
    fn from(amount: Amount) -> U128x128 {
        U128x128::from(amount.inner)
    }
}

impl From<&Amount> for U128x128 {
    fn from(value: &Amount) -> Self {
        (*value).into()
    }
}

impl TryFrom<U128x128> for Amount {
    type Error = ParserError;
    fn try_from(value: U128x128) -> Result<Self, Self::Error> {
        Ok(Amount {
            inner: value.try_into()?,
        })
    }
}

impl From<Amount> for Fq {
    fn from(val: Amount) -> Self {
        Fq::from(val.inner)
    }
}

impl From<Amount> for Fr {
    fn from(val: Amount) -> Self {
        Fr::from(val.inner)
    }
}

impl ops::Add<Amount> for Amount {
    type Output = Amount;

    fn add(self, rhs: Amount) -> Amount {
        Amount {
            inner: self.inner + rhs.inner,
        }
    }
}

#[repr(C)]
#[derive(Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct AmountC {
    pub lo: u64,
    pub hi: u64,
}
