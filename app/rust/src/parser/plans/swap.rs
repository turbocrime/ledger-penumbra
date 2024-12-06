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
    amount::Amount,
    bytes::BytesC,
    commitment::Commitment,
    effect_hash::{create_personalized_state, EffectHash},
    fee::Fee,
    swap_payload::SwapPayload,
    swap_plaintext::SwapPlaintext,
    swap_plaintext::SwapPlaintextC,
    trading_pair::TradingPair,
};
use crate::ParserError;
use decaf377::Fr;

#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct Body {
    pub trading_pair: TradingPair,
    pub delta_1_i: Amount,
    pub delta_2_i: Amount,
    pub fee_commitment: Commitment,
    pub payload: SwapPayload,
}

#[repr(C)]
#[derive(Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct SwapPlanC {
    pub has_swap_plaintext: bool,
    pub swap_plaintext: SwapPlaintextC,
    pub fee_blinding: BytesC,
    pub proof_blinding_r: BytesC,
    pub proof_blinding_s: BytesC,
}

impl SwapPlanC {
    pub fn effect_hash(&self, fvk: &FullViewingKey) -> Result<EffectHash, ParserError> {
        let body = self.swap_body(fvk);

        if let Ok(body) = body {
            let mut state = create_personalized_state("/penumbra.core.component.dex.v1.SwapBody");

            state.update(&body.trading_pair.to_proto()?);
            state.update(&[0x12]); // encode tag
            let (asset_1, len_1) = body.delta_1_i.to_proto();
            state.update(&asset_1[..len_1]);

            state.update(&[0x1a]); // encode tag
            let (asset_2, len_2) = body.delta_2_i.to_proto();
            state.update(&asset_2[..len_2]);

            state.update(&body.fee_commitment.to_proto_swap());
            state.update(&body.payload.to_proto());

            let hash = state.finalize();
            Ok(EffectHash(*hash.as_array()))
        } else {
            Err(ParserError::InvalidLength)
        }
    }

    pub fn swap_body(&self, fvk: &FullViewingKey) -> Result<Body, ParserError> {
        let fee_commitment = self.fee_commitment()?;
        let swap_plaintext = SwapPlaintext::try_from(self.swap_plaintext.clone())?;
        let payload = swap_plaintext.encrypt(fvk.outgoing())?;

        let body = Body {
            trading_pair: TradingPair::try_from(self.swap_plaintext.trading_pair.clone())?,
            delta_1_i: Amount::try_from(self.swap_plaintext.delta_1_i.clone())?,
            delta_2_i: Amount::try_from(self.swap_plaintext.delta_2_i.clone())?,
            fee_commitment,
            payload,
        };

        Ok(body)
    }

    pub fn fee_commitment(&self) -> Result<Commitment, ParserError> {
        let fee_blinding_fr = self.get_fee_blinding_fr()?;
        let fee = Fee::try_from(self.swap_plaintext.claim_fee.clone())?;
        fee.commit(fee_blinding_fr)
    }

    pub fn get_fee_blinding(&self) -> Result<&[u8], ParserError> {
        self.fee_blinding.get_bytes()
    }

    pub fn get_fee_blinding_fr(&self) -> Result<Fr, ParserError> {
        let fee_blinding_bytes = self.get_fee_blinding()?;
        Ok(Fr::from_le_bytes_mod_order(fee_blinding_bytes))
    }
}
