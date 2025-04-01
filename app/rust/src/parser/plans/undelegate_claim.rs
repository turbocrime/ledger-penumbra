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

use crate::constants::UNDELEGATE_CLAIM_PERSONALIZED;
use crate::ffi::bech32::bech32_encode;
use crate::parser::{
    amount::AmountC,
    balance::Balance,
    bytes::BytesC,
    commitment::Commitment,
    effect_hash::{create_personalized_state, EffectHash},
    id::AssetId,
    id::Id,
    identity_key::IdentityKeyC,
    penalty::{Penalty, PenaltyC},
    validator_identity::ValidatorIdentity,
};
use crate::protobuf_h::stake_pb::{
    penumbra_core_component_stake_v1_UndelegateClaimBody_balance_commitment_tag,
    penumbra_core_component_stake_v1_UndelegateClaimBody_penalty_tag,
    penumbra_core_component_stake_v1_UndelegateClaimBody_unbonding_start_height_tag,
    penumbra_core_component_stake_v1_UndelegateClaimBody_validator_identity_tag, PB_LTYPE_UVARINT,
};
use crate::utils::protobuf::{encode_and_update_proto_field, encode_and_update_proto_number};
use crate::ParserError;
use decaf377::{Fq, Fr};
use itoa::Buffer;

pub struct Body {
    /// The identity key of the validator to undelegate from.
    pub validator_identity: ValidatorIdentity,
    /// The penalty applied to undelegation, in bps^2.
    pub penalty: Penalty,
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
    pub has_penalty: bool,
    pub penalty: PenaltyC,
    pub has_unbonding_amount: bool,
    pub unbonding_amount: AmountC,
    pub balance_blinding: BytesC,
    pub unbonding_start_height: u64,
}

impl UndelegateClaimPlanC {
    pub fn effect_hash(&self) -> Result<EffectHash, ParserError> {
        let body = self.undelegate_claim_body()?;

        let mut state = create_personalized_state(
            std::str::from_utf8(UNDELEGATE_CLAIM_PERSONALIZED)
                .map_err(|_| ParserError::InvalidUtf8)?,
        );

        // encode validator identity
        let validator_identity = body.validator_identity.to_proto()?;
        encode_and_update_proto_field(
            &mut state,
            penumbra_core_component_stake_v1_UndelegateClaimBody_validator_identity_tag as u64,
            PB_LTYPE_UVARINT as u64,
            &validator_identity,
            validator_identity.len(),
        )?;

        // encode penalty
        let penalty = body.penalty.to_proto()?;
        encode_and_update_proto_field(
            &mut state,
            penumbra_core_component_stake_v1_UndelegateClaimBody_penalty_tag as u64,
            PB_LTYPE_UVARINT as u64,
            &penalty,
            penalty.len(),
        )?;

        // encode balance commitment
        let balance_commitment = body.balance_commitment.to_proto()?;
        encode_and_update_proto_field(
            &mut state,
            penumbra_core_component_stake_v1_UndelegateClaimBody_balance_commitment_tag as u64,
            PB_LTYPE_UVARINT as u64,
            &balance_commitment,
            balance_commitment.len(),
        )?;

        // encode unbonding start height
        encode_and_update_proto_number(
            &mut state,
            penumbra_core_component_stake_v1_UndelegateClaimBody_unbonding_start_height_tag as u64,
            body.unbonding_start_height,
        )?;

        Ok(EffectHash(*state.finalize().as_array()))
    }

    pub fn undelegate_claim_body(&self) -> Result<Body, ParserError> {
        let validator_identity: [u8; 32] = self
            .validator_identity
            .ik
            .get_bytes()?
            .try_into()
            .map_err(|_| ParserError::InvalidLength)?;
        let penalty = self
            .penalty
            .inner
            .get_bytes()?
            .try_into()
            .map_err(|_| ParserError::InvalidLength)?;
        let balance_commitment = self.balance()?.commit(self.get_balance_blinding_fr()?)?;

        let body = Body {
            validator_identity: ValidatorIdentity(validator_identity),
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
        penalty.balance_for_claim(self.unbonding_id()?, unbonding_amount)
    }

    pub fn unbonding_id(&self) -> Result<Id, ParserError> {
        let hrp = "penumbravalid";
        let mut validator_identity_bytes = [0u8; 72];
        bech32_encode(
            hrp,
            self.validator_identity.ik.get_bytes()?,
            &mut validator_identity_bytes,
        )
        .unwrap();

        let mut result = [0u8; 150];
        let prefix = b"uunbonding_start_at_";

        let mut buffer = Buffer::new();
        let unbonding_start_height_str = buffer.format(self.unbonding_start_height).as_bytes();
        let id_len =
            prefix.len() + unbonding_start_height_str.len() + 1 + validator_identity_bytes.len();

        result[..prefix.len()].copy_from_slice(prefix);
        result[prefix.len()..prefix.len() + unbonding_start_height_str.len()]
            .copy_from_slice(unbonding_start_height_str);
        result[prefix.len() + unbonding_start_height_str.len()] = b'_';
        result[prefix.len() + unbonding_start_height_str.len() + 1..id_len]
            .copy_from_slice(&validator_identity_bytes);

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
