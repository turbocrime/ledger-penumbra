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
    bytes::BytesC,
    identity_key::IdentityKeyC,
    amount::AmountC,
    penalty::{PenaltyC, Penalty},
    effect_hash::{create_personalized_state, EffectHash},
    commitment::Commitment,
    value::Balance,
    id::Id,
    id::AssetId,
};
use decaf377::{Fr, Fq};
use crate::ParserError;
use crate::ffi::bech32::bech32_encode;
use itoa::Buffer;
use crate::utils::protobuf::encode_varint;

pub struct Body {
    /// The identity key of the validator to undelegate from.
    pub validator_identity: [u8; 32],
    /// The penalty applied to undelegation, in bps^2.
    pub penalty: [u8; 32],
    /// The action's contribution to the transaction's value balance.
    pub balance_commitment: Commitment,
    /// The height at which unbonding started.
    pub unbonding_start_height: u64,
}

#[repr(C)]
#[derive(Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct UndelegateClaimPlanC {
    pub has_validator_identity: bool,
    pub validator_identity: IdentityKeyC,
    pub start_epoch_index: u64,
    pub has_penalty: bool,
    pub penalty: PenaltyC,
    pub has_unbonding_amount: bool,
    pub unbonding_amount: AmountC,
    pub balance_blinding: BytesC,
    pub proof_blinding_r: BytesC,
    pub proof_blinding_s: BytesC,
    pub unbonding_start_height: u64,
}

impl UndelegateClaimPlanC {
    pub fn effect_hash(&self) -> Result<EffectHash, ParserError> {
        let body = self.undelegate_claim_body()?;

        let mut state = create_personalized_state("/penumbra.core.component.stake.v1.UndelegateClaimBody");

        state.update(&[0x0a, 0x22, 0x0a, 0x20]); // encode header 0a220a20 validator_identity
        state.update(&body.validator_identity);

        state.update(&[0x1a, 0x22, 0x0a, 0x20]); // encode header 1a220a20 penalty
        state.update(&body.penalty);

        state.update(&body.balance_commitment.to_proto_unbonding_claim());

        let mut encoded = [0u8; 10];
        encoded[0] = 0x28;
        let pos = 1;
        let len = encode_varint(body.unbonding_start_height, &mut encoded[pos..]);
        state.update(&encoded[..len + 1]);

        let hash = state.finalize();
        Ok(EffectHash(*hash.as_array()))
    }

    pub fn undelegate_claim_body(&self) -> Result<Body, ParserError> {
        let validator_identity: [u8; 32] = self.validator_identity.ik.get_bytes()?.try_into().map_err(|_| ParserError::InvalidLength)?;
        let penalty = self.penalty.inner.get_bytes()?.try_into().map_err(|_| ParserError::InvalidLength)?;
        let balance_commitment = self.balance()?.commit(self.get_balance_blinding_fr()?)?;

        let body = Body {
            validator_identity,
            penalty,
            balance_commitment,
            unbonding_start_height: self.unbonding_start_height,
        };

        Ok(body)
    }

    pub fn get_balance_blinding(&self) -> Result<&[u8], ParserError> {
        self.balance_blinding.get_bytes()
    }

    pub fn get_balance_blinding_fr(&self) -> Result<Fr, ParserError> {
        let balance_blinding_bytes = self.get_balance_blinding()?;
        Ok(Fr::from_le_bytes_mod_order(balance_blinding_bytes))
    }

    pub fn balance(&self) -> Result<Balance, ParserError> {
        let penalty = Penalty::try_from(self.penalty.clone())?;
        let unbonding_amount = self.unbonding_amount.clone().try_into()?;
        penalty
            .balance_for_claim(self.unbonding_id()?, unbonding_amount)
    }

    pub fn unbonding_id(&self) -> Result<Id, ParserError> {
        let hrp = "penumbravalid";
        let mut validator_identity_bytes = [0u8; 72];
        bech32_encode(hrp, self.validator_identity.ik.get_bytes()?, &mut validator_identity_bytes).unwrap();
    
        let mut result = [0u8; 150];
        let prefix = b"uunbonding_start_at_";
    
        let mut buffer = Buffer::new();
        let unbonding_start_height_str = buffer.format(self.unbonding_start_height).as_bytes();
        let id_len = prefix.len() + unbonding_start_height_str.len() + 1 + validator_identity_bytes.len();
    
        result[..prefix.len()].copy_from_slice(prefix);
        result[prefix.len()..prefix.len() + unbonding_start_height_str.len()].copy_from_slice(unbonding_start_height_str);
        result[prefix.len() + unbonding_start_height_str.len()] = b'_';
        result[prefix.len() + unbonding_start_height_str.len() + 1..id_len].copy_from_slice(&validator_identity_bytes);
    
        unsafe {
            let data_slice = std::slice::from_raw_parts(result.as_ptr(), id_len);
            if let Ok(asset) = AssetId::new(std::str::from_utf8(data_slice).unwrap()) {
                Ok(Id(Fq::from_le_bytes_mod_order(&asset.to_bytes())))
            } else {
                Err(ParserError::InvalidAssetId)
            }
        }
    }
}
