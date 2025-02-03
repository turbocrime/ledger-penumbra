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

use crate::keys::FullViewingKey;
use crate::parser::{
    amount::{Amount, AmountC},
    bytes::BytesC,
    effect_hash::{create_personalized_state, EffectHash},
    note::{Note, NoteC},
    nullifier::Nullifier,
    value::Value,
};
use crate::utils::protobuf::encode_varint;
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
    pub rk: VerificationKey<SpendAuth>,
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

        let mut state =
            create_personalized_state("/penumbra.core.component.governance.v1.DelegatorVoteBody");

        // proposal
        let mut encoded = [0u8; 11];
        encoded[0] = 0x08;
        let mut pos = 1;
        let mut len = encode_varint(body.proposal, &mut encoded[pos..]);
        state.update(&encoded[..len + 1]);

        // start_position
        if body.start_position > 0 {
            encoded[0] = 0x10;
            pos = 1;
            len = encode_varint(body.start_position, &mut encoded[pos..]);
            state.update(&encoded[..len + 1]);
        }

        // vote
        state.update(&[0x1a, 0x02]);
        encoded[0] = 0x08;
        pos = 1;
        len = encode_varint(body.vote as u64, &mut encoded[pos..]);
        state.update(&encoded[..len + 1]);

        // value amount
        state.update(&[0x22]);
        let (value, value_len) = body.value.to_proto();
        state.update(&value[..value_len]);

        // unbonded_amount
        state.update(&[0x2a]); // encode tag
        let (unbonded_amount, unbonded_amount_len) = body.unbonded_amount.to_proto();
        state.update(&unbonded_amount[..unbonded_amount_len]);

        // nullifier
        state.update(&body.nullifier.to_proto());

        // rk
        state.update(&[0x3a, 0x22, 0x0a, 0x20]);
        state.update(&body.rk.to_bytes());

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
            rk: self.rk(fvk)?,
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
