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

use crate::constants::ACTION_DUTCH_AUCTION_WITHDRAWAL_PERSONALIZED;
use crate::parser::{
    balance::Balance,
    commitment::Commitment,
    effect_hash::{create_personalized_state, EffectHash},
    id::{IdC, IdRaw},
    value::{Imbalance, Sign, Value, ValueC},
};
use crate::protobuf_h::auction_pb::{
    penumbra_core_component_auction_v1_ActionDutchAuctionWithdraw_auction_id_tag,
    penumbra_core_component_auction_v1_ActionDutchAuctionWithdraw_reserves_commitment_tag,
    penumbra_core_component_auction_v1_ActionDutchAuctionWithdraw_seq_tag, PB_LTYPE_UVARINT,
};
use crate::utils::protobuf::{encode_and_update_proto_field, encode_and_update_proto_number};
use crate::ParserError;
use decaf377::Fr;

pub struct ActionDutchAuctionWithdraw {
    pub auction_id: IdRaw,
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
            std::str::from_utf8(ACTION_DUTCH_AUCTION_WITHDRAWAL_PERSONALIZED)
                .map_err(|_| ParserError::InvalidUtf8)?,
        );

        // auction_id
        let auction_id = action_dutch_auction_withdraw.auction_id.to_proto()?;
        encode_and_update_proto_field(
            &mut state,
            penumbra_core_component_auction_v1_ActionDutchAuctionWithdraw_auction_id_tag as u64,
            PB_LTYPE_UVARINT as u64,
            &auction_id,
            auction_id.len(),
        )?;

        // sequence
        encode_and_update_proto_number(
            &mut state,
            penumbra_core_component_auction_v1_ActionDutchAuctionWithdraw_seq_tag as u64,
            action_dutch_auction_withdraw.seq,
        )?;

        // reserves_commitment
        let reserves_commitment = action_dutch_auction_withdraw
            .reserves_commitment
            .to_proto()?;
        encode_and_update_proto_field(
            &mut state,
            penumbra_core_component_auction_v1_ActionDutchAuctionWithdraw_reserves_commitment_tag
                as u64,
            PB_LTYPE_UVARINT as u64,
            &reserves_commitment,
            reserves_commitment.len(),
        )?;

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
            auction_id: IdRaw(auction_id),
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
