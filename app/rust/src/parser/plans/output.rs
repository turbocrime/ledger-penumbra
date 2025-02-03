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

use crate::address::Address;
use crate::keys::FullViewingKey;
use crate::parser::note::Note;
use crate::parser::{
    address::AddressC,
    balance::Balance,
    bytes::BytesC,
    commitment::Commitment,
    effect_hash::{create_personalized_state, EffectHash},
    note_payload::NotePayload,
    rseed::Rseed,
    symmetric::PayloadKey,
    symmetric::{OvkWrappedKey, WrappedMemoKey},
    value::{Imbalance, Sign, Value, ValueC},
};
use crate::ParserError;
use decaf377::Fr;

#[derive(Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct Body {
    pub note_payload: NotePayload,
    pub balance_commitment: Commitment,
    pub ovk_wrapped_key: OvkWrappedKey,
    pub wrapped_memo_key: WrappedMemoKey,
}

#[repr(C)]
#[derive(Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct OutputPlanC {
    pub value: ValueC,
    pub dest_address: AddressC,
    pub rseed: BytesC,
    pub value_blinding: BytesC,
}

impl OutputPlanC {
    pub fn effect_hash(
        &self,
        fvk: &FullViewingKey,
        memo_key: &[u8],
    ) -> Result<EffectHash, ParserError> {
        let memo_payload_key = PayloadKey::from_bytes(memo_key);
        let body = self.body(fvk, &memo_payload_key);

        if let Ok(body) = body {
            let mut state =
                create_personalized_state("/penumbra.core.component.shielded_pool.v1.OutputBody");

            state.update(&body.note_payload.to_proto());
            state.update(&body.balance_commitment.to_proto_output());
            state.update(&body.wrapped_memo_key.to_proto());
            state.update(&body.ovk_wrapped_key.to_proto());

            Ok(EffectHash(*state.finalize().as_array()))
        } else {
            Err(ParserError::InvalidLength)
        }
    }

    pub fn body(&self, fvk: &FullViewingKey, memo_key: &PayloadKey) -> Result<Body, ParserError> {
        let ovk = fvk.outgoing();
        let note = self.output_note()?;
        let value = self.balance()?;
        let balance_commitment = value.commit(self.get_value_blinding_fr()?)?;

        // Encrypt the note to the recipient...
        let esk = note.ephemeral_secret_key()?;

        // ... and wrap the encryption key to ourselves.
        let ovk_wrapped_key = note.encrypt_key(ovk, balance_commitment.clone())?;

        let wrapped_memo_key = WrappedMemoKey::encrypt(
            memo_key,
            esk,
            note.transmission_key(),
            &note.diversified_generator()?,
        )?;

        Ok(Body {
            note_payload: note.payload()?,
            balance_commitment,
            ovk_wrapped_key,
            wrapped_memo_key,
        })
    }

    pub fn output_note(&self) -> Result<Note, ParserError> {
        let value = Value::try_from(self.value.clone())?;
        let rseed = Rseed::try_from(self.rseed.clone())?;
        let address = Address::try_from(self.dest_address.inner.get_bytes()?)?;

        Note::from_parts(address, value, rseed)
    }

    pub fn balance(&self) -> Result<Balance, ParserError> {
        let mut balance = Balance::new();
        balance.insert(Imbalance {
            value: Value::try_from(self.value.clone())?,
            sign: Sign::Required,
        })?;
        Ok(balance)
    }

    pub fn get_rseed(&self) -> Result<&[u8], ParserError> {
        self.rseed.get_bytes()
    }

    pub fn get_value_blinding(&self) -> Result<&[u8], ParserError> {
        self.value_blinding.get_bytes()
    }

    pub fn get_value_blinding_fr(&self) -> Result<Fr, ParserError> {
        let value_blinding_bytes = self.get_value_blinding()?;
        Ok(Fr::from_le_bytes_mod_order(value_blinding_bytes))
    }
}
