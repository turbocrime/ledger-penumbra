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

use crate::constants::DELEGATOR_VOTE_PERSONALIZED;
use crate::keys::FullViewingKey;
use crate::parser::{
    amount::{Amount, AmountC},
    bytes::BytesC,
    effect_hash::{create_personalized_state, EffectHash},
    note::{Note, NoteC},
    nullifier::Nullifier,
    rk::Rk,
    value::Value,
};
use crate::protobuf_h::governance_pb::{
    penumbra_core_component_governance_v1_DelegatorVoteBody_nullifier_tag,
    penumbra_core_component_governance_v1_DelegatorVoteBody_proposal_tag,
    penumbra_core_component_governance_v1_DelegatorVoteBody_rk_tag,
    penumbra_core_component_governance_v1_DelegatorVoteBody_start_position_tag,
    penumbra_core_component_governance_v1_DelegatorVoteBody_unbonded_amount_tag,
    penumbra_core_component_governance_v1_DelegatorVoteBody_value_tag,
    penumbra_core_component_governance_v1_DelegatorVoteBody_vote_tag,
    penumbra_core_component_governance_v1_Vote_vote_tag, PB_LTYPE_UVARINT,
};
use crate::utils::protobuf::{
    encode_and_update_proto_field, encode_and_update_proto_number, encode_proto_number,
    encode_varint,
};
use crate::ParserError;
use decaf377::Fr;
use decaf377_rdsa::{SpendAuth, VerificationKey};

pub struct Body {
    /// The proposal ID the vote is for.
    pub proposal: u64,
    /// The start position of the proposal in the TCT.
    pub start_position: u64,
    /// The vote on the proposal.
    pub vote: u8, // With flow encryption, this will be a triple of flow ciphertexts
    /// The value of the staked note being used to vote.
    pub value: Value, // With flow encryption, this will be a triple of balance commitments, and a public denomination
    /// The unbonded amount equivalent to the value above
    pub unbonded_amount: Amount,
    /// The nullifier of the staked note being used to vote.
    pub nullifier: Nullifier,
    /// The randomized validating key for the spend authorization signature.
    pub rk: Rk,
}

#[repr(C)]
#[derive(Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct DelegatorVotePlanC {
    pub proposal: u64,
    pub start_position: u64,
    pub has_vote: bool,
    pub vote: u8,
    pub has_staked_note: bool,
    pub staked_note: NoteC,
    pub staked_note_position: u64,
    pub has_unbonded_amount: bool,
    pub unbonded_amount: AmountC,
    pub randomizer: BytesC,
}

impl DelegatorVotePlanC {
    pub fn effect_hash(&self, fvk: &FullViewingKey) -> Result<EffectHash, ParserError> {
        let body = self.delegator_vote_body(fvk)?;

        let mut state = create_personalized_state(
            std::str::from_utf8(DELEGATOR_VOTE_PERSONALIZED)
                .map_err(|_| ParserError::InvalidUtf8)?,
        );

        // proposal
        encode_and_update_proto_number(
            &mut state,
            penumbra_core_component_governance_v1_DelegatorVoteBody_proposal_tag as u64,
            body.proposal,
        )?;

        // start_position
        if body.start_position > 0 {
            encode_and_update_proto_number(
                &mut state,
                penumbra_core_component_governance_v1_DelegatorVoteBody_start_position_tag as u64,
                body.start_position,
            )?;
        }

        // vote
        state.update(&[
            ((penumbra_core_component_governance_v1_DelegatorVoteBody_vote_tag << 3) | 2) as u8,
        ]);
        let mut vote_buf = [0u8; 20];
        let len = encode_proto_number(
            penumbra_core_component_governance_v1_Vote_vote_tag as u64,
            body.vote as u64,
            &mut vote_buf,
        )?;
        let mut vote_size_buf = [0u8; 10];
        let vote_size_buf_len = encode_varint(len as u64, &mut vote_size_buf)?;
        state.update(&vote_size_buf[..vote_size_buf_len]);
        state.update(&vote_buf[..len]);

        // value amount
        let (value, value_len) = body.value.to_proto()?;
        encode_and_update_proto_field(
            &mut state,
            penumbra_core_component_governance_v1_DelegatorVoteBody_value_tag as u64,
            PB_LTYPE_UVARINT as u64,
            &value,
            value_len,
        )?;

        // unbonded_amount
        state.update(&[
            ((penumbra_core_component_governance_v1_DelegatorVoteBody_unbonded_amount_tag << 3) | 2)
                as u8,
        ]);
        let (unbonded_amount, unbonded_amount_len) = body.unbonded_amount.to_proto()?;
        state.update(&unbonded_amount[..unbonded_amount_len]);

        // nullifier
        encode_and_update_proto_field(
            &mut state,
            penumbra_core_component_governance_v1_DelegatorVoteBody_nullifier_tag as u64,
            PB_LTYPE_UVARINT as u64,
            &body.nullifier.to_proto()?,
            body.nullifier.to_proto()?.len(),
        )?;

        // rk
        let rk = body.rk.to_proto()?;
        encode_and_update_proto_field(
            &mut state,
            penumbra_core_component_governance_v1_DelegatorVoteBody_rk_tag as u64,
            PB_LTYPE_UVARINT as u64,
            &rk,
            rk.len(),
        )?;

        Ok(EffectHash(*state.finalize().as_array()))
    }

    pub fn delegator_vote_body(&self, fvk: &FullViewingKey) -> Result<Body, ParserError> {
        let value = Value::try_from(self.staked_note.value.clone())?;
        let unbonded_amount = Amount::try_from(self.unbonded_amount.clone())?;

        let nk = fvk.nullifier_key();
        let note = Note::try_from(self.staked_note.clone())?;
        let nullifier = Nullifier::derive(nk, self.staked_note_position, &note.commit()?.0);

        let body = Body {
            proposal: self.proposal,
            start_position: self.start_position,
            vote: self.vote,
            value,
            unbonded_amount,
            nullifier,
            rk: Rk(self.rk(fvk)?),
        };

        Ok(body)
    }

    pub fn rk(&self, fvk: &FullViewingKey) -> Result<VerificationKey<SpendAuth>, ParserError> {
        Ok(fvk
            .spend_verification_key()
            .randomize(&self.get_randomizer_fr()?))
    }

    pub fn get_randomizer(&self) -> Result<&[u8], ParserError> {
        self.randomizer.get_bytes()
    }

    pub fn get_randomizer_fr(&self) -> Result<Fr, ParserError> {
        let randomizer_bytes = self.get_randomizer()?;
        Ok(Fr::from_le_bytes_mod_order(randomizer_bytes))
    }
}
