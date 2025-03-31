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

use crate::constants::SWAP_PERSONALIZED;
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
use crate::protobuf_h::dex_pb::{
    penumbra_core_component_dex_v1_SwapBody_delta_1_i_tag,
    penumbra_core_component_dex_v1_SwapBody_delta_2_i_tag,
    penumbra_core_component_dex_v1_SwapBody_fee_commitment_tag,
    penumbra_core_component_dex_v1_SwapBody_payload_tag,
    penumbra_core_component_dex_v1_SwapBody_trading_pair_tag, PB_LTYPE_UVARINT,
};
use crate::utils::protobuf::encode_and_update_proto_field;
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
}

impl SwapPlanC {
    pub fn effect_hash(&self, fvk: &FullViewingKey) -> Result<EffectHash, ParserError> {
        let body = self.swap_body(fvk)?;

        let mut state = create_personalized_state(
            std::str::from_utf8(SWAP_PERSONALIZED).map_err(|_| ParserError::InvalidUtf8)?,
        );

        // encode trading pair
        let trading_pair = body.trading_pair.to_proto()?;
        encode_and_update_proto_field(
            &mut state,
            penumbra_core_component_dex_v1_SwapBody_trading_pair_tag as u64,
            PB_LTYPE_UVARINT as u64,
            &trading_pair,
            trading_pair.len(),
        )?;

        // encode delta_1_i
        state.update(&[
            ((penumbra_core_component_dex_v1_SwapBody_delta_1_i_tag << 3) | 2) as u8,
        ]);
        let (asset_1, len_1) = body.delta_1_i.to_proto()?;
        state.update(&asset_1[..len_1]);

        // encode delta_2_i
        state.update(&[((penumbra_core_component_dex_v1_SwapBody_delta_2_i_tag << 3) | 2) as u8]);
        let (asset_2, len_2) = body.delta_2_i.to_proto()?;
        state.update(&asset_2[..len_2]);

        // encode fee_commitment
        let fee_commitment = body.fee_commitment.to_proto()?;
        encode_and_update_proto_field(
            &mut state,
            penumbra_core_component_dex_v1_SwapBody_fee_commitment_tag as u64,
            PB_LTYPE_UVARINT as u64,
            &fee_commitment,
            fee_commitment.len(),
        )?;

        // encode payload
        let payload = body.payload.to_proto()?;
        encode_and_update_proto_field(
            &mut state,
            penumbra_core_component_dex_v1_SwapBody_payload_tag as u64,
            PB_LTYPE_UVARINT as u64,
            &payload,
            payload.len(),
        )?;

        Ok(EffectHash(*state.finalize().as_array()))
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
