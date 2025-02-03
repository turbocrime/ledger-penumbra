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

use crate::constants::MAX_REWARDS;
use crate::parser::{
    commitment::Commitment,
    effect_hash::{create_personalized_state, EffectHash},
    id::IdC,
    reserves::{Reserves, ReservesC},
    trading_pair::{TradingPair, TradingPairC},
    value::{Sign, Value, ValueC},
};
use crate::utils::protobuf::encode_varint;
use crate::ParserError;
use decaf377::Fr;

pub struct PositionWithdraw {
    /// The identity key of the validator to undelegate from.
    pub position_id: [u8; 32],
    /// A transparent (zero blinding factor) commitment to the position's final reserves and fees.
    ///
    /// The chain will check this commitment by recomputing it with the on-chain state.
    pub reserves_commitment: Commitment,
    /// The sequence number of the withdrawal, allowing multiple withdrawals from the same position.
    pub sequence: u64,
}

#[repr(C)]
#[derive(Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct PositionWithdrawPlanC {
    pub has_reserves: bool,
    pub reserves: ReservesC,
    pub has_position_id: bool,
    pub position_id: IdC,
    pub has_pair: bool,
    pub pair: TradingPairC,
    pub sequence: u64,
    pub rewards: [ValueC; MAX_REWARDS],
    pub rewards_qty: u8,
}

impl PositionWithdrawPlanC {
    pub fn effect_hash(&self) -> Result<EffectHash, ParserError> {
        let position_withdraw = self.position_withdraw()?;

        let mut state =
            create_personalized_state("/penumbra.core.component.dex.v1.PositionWithdraw");

        // position_id
        state.update(&[0x0a, 0x22, 0x0a, 0x20]);
        state.update(&position_withdraw.position_id);

        // reserves_commitment
        state.update(
            &position_withdraw
                .reserves_commitment
                .to_proto_position_withdraw(),
        );

        // sequence
        let mut encoded = [0u8; 11];
        encoded[0] = 0x18;
        let pos = 1;
        let len = encode_varint(position_withdraw.sequence, &mut encoded[pos..]);
        state.update(&encoded[..len + 1]);

        Ok(EffectHash(*state.finalize().as_array()))
    }

    pub fn position_withdraw(&self) -> Result<PositionWithdraw, ParserError> {
        let position_id: [u8; 32] = self
            .position_id
            .inner
            .get_bytes()?
            .try_into()
            .map_err(|_| ParserError::InvalidLength)?;

        let reserves_commitment = self.reserves_commitment()?;

        let position_withdraw = PositionWithdraw {
            position_id,
            reserves_commitment,
            sequence: self.sequence,
        };

        Ok(position_withdraw)
    }

    pub fn reserves_commitment(&self) -> Result<Commitment, ParserError> {
        let reserves = Reserves::try_from(self.reserves.clone())?;
        let trading_pair = TradingPair::try_from(self.pair.clone())?;

        let mut reserves_balance = reserves.balance(&trading_pair)?;

        for i in 0..self.rewards_qty as usize {
            let value = Value::try_from(self.rewards[i].clone())?;
            reserves_balance = reserves_balance.add(&value, Sign::Provided)?;
        }

        reserves_balance.commit(Fr::ZERO)
    }
}
