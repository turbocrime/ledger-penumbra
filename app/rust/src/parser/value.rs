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
use crate::parser::{
    amount::{Amount, AmountC},
    commitment::Commitment,
    id::{Id, IdC},
    ParserError,
};
use decaf377::Fr;
use crate::utils::protobuf::encode_varint;

#[derive(Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub enum Sign {
    Required,
    Provided,
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

// Only two imbalances are supported for now
const IMBALANCES_SIZE: usize = 2;

#[derive(Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct Balance {
    pub imbalances: [Option<Imbalance>; IMBALANCES_SIZE],
}

impl Balance {
    pub fn new() -> Self {
        Balance {
            imbalances: [None, None],
        }
    }

    pub fn add(&mut self, imbalance: Imbalance) -> Result<(), ParserError> {
        for slot in &mut self.imbalances {
            if slot.is_none() {
                *slot = Some(imbalance);
                return Ok(());
            }
        }
        Err(ParserError::InvalidLength)
    }

    pub fn commit(&self, blinding_factor: Fr) -> Result<Commitment, ParserError> {
        if !self.has_valid_imbalance() {
            return Err(ParserError::InvalidLength);
        }
    
        let mut commitment = decaf377::Element::IDENTITY;
    
        for imbalance in self.imbalances.iter().flatten() {
            let g_v = imbalance.value.asset_id.value_generator();
            let amount_fr: Fr = Into::into(imbalance.value.amount);
    
            if amount_fr.ne(&Fr::ZERO) {
                match imbalance.sign {
                    Sign::Required => {
                        commitment -= g_v * amount_fr;
                    }
                    Sign::Provided => {
                        commitment += g_v * amount_fr;
                    }
                }
            }
        }
    
        let value_blinding_generator = Commitment::value_blinding_generator();
        commitment += blinding_factor * value_blinding_generator;
    
        Ok(commitment.into())
    }

    fn has_valid_imbalance(&self) -> bool {
        self.imbalances.iter().any(|slot| slot.is_some())
    }
}

impl Default for Balance {
    fn default() -> Self {
        Self::new()
    }
}

impl Value {
    pub const LEN: usize = AMOUNT_LEN_BYTES + ID_LEN_BYTES;

    pub fn to_bytes(&self) -> Result<[u8; Self::LEN], ParserError> {
        let mut bytes = [0; Self::LEN];
        bytes[0..AMOUNT_LEN_BYTES].copy_from_slice(&self.amount.to_le_bytes());
        bytes[AMOUNT_LEN_BYTES..AMOUNT_LEN_BYTES + ID_LEN_BYTES]
            .copy_from_slice(&self.asset_id.to_bytes());
        Ok(bytes)
    }

    pub fn to_proto(&self) -> ([u8; 62], usize) {
        let (value_amount, value_amount_len) = self.amount.to_proto();

        // Calculate the total length of the value
        let value_len = 1 + value_amount_len + Id::PROTO_LEN;
        
        // Encode the length as a varint
        let mut value_len_encoded = [0u8; 10];
        let len = encode_varint(value_len as u64, &mut value_len_encoded);

        // Initialize the proto buffer
        let mut proto = [0u8; 62];
        
        // Copy the encoded length into the proto buffer
        proto[..len].copy_from_slice(&value_len_encoded[..len]);

        // Add the tag
        proto[len] = 0x0a;

        // Copy the value amount into the proto buffer
        proto[len + 1..len + 1 + value_amount_len]
            .copy_from_slice(&value_amount[..value_amount_len]);

        // Copy the asset ID into the proto buffer
        proto[len + 1 + value_amount_len..len + 1 + value_amount_len + Id::PROTO_LEN]
            .copy_from_slice(&self.asset_id.to_proto());

        (proto, len + 1 + value_amount_len + Id::PROTO_LEN)
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
