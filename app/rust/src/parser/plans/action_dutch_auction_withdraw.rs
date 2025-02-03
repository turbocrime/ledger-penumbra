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

use crate::parser::{
    balance::Balance,
    commitment::Commitment,
    effect_hash::{create_personalized_state, EffectHash},
    id::IdC,
    value::{Imbalance, Sign, Value, ValueC},
};
use crate::utils::protobuf::encode_varint;
use crate::ParserError;
use decaf377::Fr;

pub struct ActionDutchAuctionWithdraw {
    pub auction_id: [u8; 32],
    pub seq: u64,
    pub reserves_commitment: Commitment,
}

#[repr(C)]
#[derive(Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct ActionDutchAuctionWithdrawPlanC {
    pub has_auction_id: bool,
    pub auction_id: IdC,
    pub seq: u64,
    pub has_reserves_input: bool,
    pub reserves_input: ValueC,
    pub has_reserves_output: bool,
    pub reserves_output: ValueC,
}

impl ActionDutchAuctionWithdrawPlanC {
    pub fn effect_hash(&self) -> Result<EffectHash, ParserError> {
        let action_dutch_auction_withdraw = self.to_action()?;

        let mut state = create_personalized_state(
            "/penumbra.core.component.auction.v1.ActionDutchAuctionWithdraw",
        );

        // auction_id
        state.update(&[0x0a, 0x22, 0x0a, 0x20]);
        state.update(&action_dutch_auction_withdraw.auction_id);

        // sequence
        let mut encoded = [0u8; 11];
        encoded[0] = 0x10;
        let pos = 1;
        let len = encode_varint(action_dutch_auction_withdraw.seq, &mut encoded[pos..]);
        state.update(&encoded[..len + 1]);

        // reserves_commitment
        state.update(
            &action_dutch_auction_withdraw
                .reserves_commitment
                .to_proto_action_dutch_auction_withdraw(),
        );

        Ok(EffectHash(*state.finalize().as_array()))
    }

    pub fn to_action(&self) -> Result<ActionDutchAuctionWithdraw, ParserError> {
        let auction_id: [u8; 32] = self
            .auction_id
            .inner
            .get_bytes()?
            .try_into()
            .map_err(|_| ParserError::InvalidLength)?;

        let reserves_commitment = self.reserves_commitment()?;

        let action_dutch_auction_withdraw = ActionDutchAuctionWithdraw {
            auction_id,
            seq: self.seq,
            reserves_commitment,
        };

        Ok(action_dutch_auction_withdraw)
    }

    pub fn reserves_balance(&self) -> Result<Balance, ParserError> {
        let mut balance = Balance::new();

        let reserves_input = Value::try_from(self.reserves_input.clone())?;
        balance.insert(Imbalance {
            value: reserves_input,
            sign: Sign::Provided,
        })?;

        let reserves_output = Value::try_from(self.reserves_output.clone())?;
        balance = balance.add(&reserves_output, Sign::Provided)?;

        Ok(balance)
    }

    pub fn reserves_commitment(&self) -> Result<Commitment, ParserError> {
        self.reserves_balance()?.commit(Fr::ZERO)
    }
}
