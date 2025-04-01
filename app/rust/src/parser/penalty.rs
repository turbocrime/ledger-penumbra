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
use crate::constants::PENALTY_BYTES;
use crate::parser::bytes::BytesC;
use crate::parser::{
    amount::Amount,
    balance::Balance,
    fee::STAKING_TOKEN_ASSET_ID_BYTES,
    fixpoint::U128x128,
    id::Id,
    value::{Imbalance, Sign, Value},
    ParserError,
};
use crate::protobuf_h::stake_pb::{
    penumbra_core_component_stake_v1_Penalty_inner_tag, PB_LTYPE_UVARINT,
};
use crate::utils::protobuf::encode_proto_field;
use decaf377::Fq;

#[derive(Copy, Clone)]
pub struct Penalty(U128x128);

impl Penalty {
    pub const PROTO_LEN: usize = PENALTY_BYTES + 2;

    /// Apply this `Penalty` to an `Amount` of unbonding tokens.
    pub fn apply_to_amount(&self, amount: Amount) -> Result<Amount, ParserError> {
        self.0.apply_to_amount(&amount)
    }

    /// Helper method to compute the effect of an UndelegateClaim on the
    /// transaction's value balance, used in planning and (transparent) proof
    /// verification.
    ///
    /// This method takes the `unbonding_id` rather than the `UnbondingToken` so
    /// that it can be used in mock proof verification, where computation of the
    /// unbonding token's asset ID happens outside of the circuit.
    pub fn balance_for_claim(
        &self,
        unbonding_id: Id,
        unbonding_amount: Amount,
    ) -> Result<Balance, ParserError> {
        // The undelegate claim action subtracts the unbonding amount and adds
        // the unbonded amount from the transaction's value balance.
        let mut balance = Balance::new();
        balance.insert(Imbalance {
            value: Value {
                amount: unbonding_amount,
                asset_id: unbonding_id,
            },
            sign: Sign::Required,
        })?;
        balance.insert(Imbalance {
            value: Value {
                amount: self.apply_to_amount(unbonding_amount)?,
                asset_id: Id(Fq::from_le_bytes_mod_order(&STAKING_TOKEN_ASSET_ID_BYTES)),
            },
            sign: Sign::Provided,
        })?;
        Ok(balance)
    }

    pub fn to_proto(&self) -> Result<[u8; Self::PROTO_LEN], ParserError> {
        let mut proto = [0u8; Self::PROTO_LEN];

        let bytes = self.0.to_bytes();
        let len = encode_proto_field(
            penumbra_core_component_stake_v1_Penalty_inner_tag as u64,
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

#[repr(C)]
#[derive(Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct PenaltyC {
    pub inner: BytesC,
}

impl<'a> TryFrom<&'a [u8]> for Penalty {
    type Error = ParserError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        U128x128::try_from(value)
            .map(Self)
            .map_err(|_| ParserError::InvalidLength)
    }
}

impl TryFrom<PenaltyC> for Penalty {
    type Error = ParserError;
    fn try_from(value: PenaltyC) -> Result<Self, Self::Error> {
        let bytes = value.inner.get_bytes()?;
        Penalty::try_from(bytes)
    }
}
