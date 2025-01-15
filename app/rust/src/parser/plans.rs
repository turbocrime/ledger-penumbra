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
    action::ActionPlan, action::ActionsHashC, detection::DetectionDataPlanC, memo::MemoPlanC,
};

use crate::constants::EFFECT_HASH_LEN;
use crate::ffi::c_api::c_fvk_bytes;
use crate::parser::bytes::BytesC;
use crate::parser::effect_hash::EffectHash;
use crate::parser::parameters::ParametersHash;
use crate::ParserError;

pub mod delegator_vote;
pub mod output;
pub mod spend;
pub mod swap;
pub mod undelegate_claim;

#[repr(C)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct TransactionPlanC {
    pub actions_hashes: ActionsHashC,
    pub has_parameters: bool,
    pub parameters_hash: ParametersHash,
    pub has_memo: bool,
    pub memo: MemoPlanC,
    pub has_detection_data: bool,
    pub detection_data: DetectionDataPlanC,
}

impl TransactionPlanC {
    pub fn effect_hash(&self) -> Result<EffectHash, ParserError> {
        let mut state = blake2b_simd::Params::new()
            .personal(b"PenumbraEfHs")
            .to_state();

        state.update(&self.parameters_hash.0);
        state.update(self.memo.effect_hash()?.as_array());
        state.update(self.detection_data.effect_hash()?.as_array());

        let num_actions = self.actions_hashes.qty as u32;
        state.update(&num_actions.to_le_bytes());

        for i in 0..num_actions {
            let action_hash = self.actions_hashes.hashes[i as usize].0;
            state.update(&action_hash);
        }

        Ok(EffectHash::from_array(*state.finalize().as_array()))
    }
}

#[no_mangle]
/// Use to compute an address and write it back into output
/// argument.
pub unsafe extern "C" fn rs_compute_effect_hash(
    plan: &TransactionPlanC,
    output: *mut u8,
    output_len: usize,
) -> u32 {
    crate::zlog("rs_compute_effect_hash\x00");
    let output = std::slice::from_raw_parts_mut(output, output_len);

    if output.len() < EFFECT_HASH_LEN {
        return ParserError::UnexpectedData as u32;
    }

    let plan_hash_result = plan.effect_hash();
    if let Ok(plan_hash) = plan_hash_result {
        let plan_hash_array = plan_hash.as_array();
        let copy_len: usize = core::cmp::min(output.len(), plan_hash_array.len());
        output[..copy_len].copy_from_slice(&plan_hash_array[..copy_len]);
    } else {
        return ParserError::UnexpectedError as u32;
    }

    ParserError::Ok as u32
}

#[no_mangle]
/// Use to compute an address and write it back into output
/// argument.
pub unsafe extern "C" fn rs_parameter_hash(
    data: &BytesC,
    output: *mut u8,
    output_len: usize,
) -> u32 {
    crate::zlog("rs_parameter_hash\x00");
    let output = std::slice::from_raw_parts_mut(output, output_len);

    if output.len() < 64 {
        return ParserError::Ok as u32;
    }

    let effect_hash: EffectHash;
    if let Ok(data_to_hash) = data.get_bytes() {
        effect_hash = EffectHash::from_proto_effecting_data(
            "/penumbra.core.transaction.v1.TransactionParameters",
            data_to_hash,
        );

        let body_hash_array = effect_hash.as_bytes();
        let copy_len: usize = core::cmp::min(output.len(), body_hash_array.len());
        output[..copy_len].copy_from_slice(&body_hash_array[..copy_len]);
    } else {
        return ParserError::EffectHashError as u32;
    }

    ParserError::Ok as u32
}

#[no_mangle]
/// Use to compute an address and write it back into output
/// argument.
pub unsafe extern "C" fn rs_spend_action_hash(
    plan: &spend::SpendPlanC,
    output: *mut u8,
    output_len: usize,
) -> u32 {
    crate::zlog("rs_spend_action_hash\x00");
    let output = std::slice::from_raw_parts_mut(output, output_len);

    if output.len() < 64 {
        return ParserError::Ok as u32;
    }

    let Ok(fvk) = c_fvk_bytes() else {
        return ParserError::InvalidFvk as u32;
    };
    let body_hash_bytes = plan.effect_hash(&fvk);

    if let Ok(body_hash_bytes) = body_hash_bytes {
        let body_hash_array = body_hash_bytes.as_array();
        let copy_len: usize = core::cmp::min(output.len(), body_hash_array.len());
        output[..copy_len].copy_from_slice(&body_hash_array[..copy_len]);
    } else {
        return ParserError::SpendPlanError as u32;
    }

    ParserError::Ok as u32
}

#[no_mangle]
/// Use to compute an address and write it back into output
/// argument.
pub unsafe extern "C" fn rs_output_action_hash(
    plan: &output::OutputPlanC,
    memo_key: &BytesC,
    output: *mut u8,
    output_len: usize,
) -> u32 {
    crate::zlog("rs_output_action_hash\x00");
    let output = std::slice::from_raw_parts_mut(output, output_len);

    if output.len() < 64 {
        return ParserError::Ok as u32;
    }

    let Ok(fvk) = c_fvk_bytes() else {
        return ParserError::InvalidFvk as u32;
    };

    let memo_key_bytes = memo_key.get_bytes().unwrap_or(&[0u8; 32]);

    let body_hash_bytes = plan.effect_hash(&fvk, memo_key_bytes);

    if let Ok(body_hash_bytes) = body_hash_bytes {
        let body_hash_array = body_hash_bytes.as_array();
        let copy_len: usize = core::cmp::min(output.len(), body_hash_array.len());
        output[..copy_len].copy_from_slice(&body_hash_array[..copy_len]);
    } else {
        return ParserError::OutputPlanError as u32;
    }

    ParserError::Ok as u32
}

#[no_mangle]
/// Use to compute an address and write it back into output
/// argument.
pub unsafe extern "C" fn rs_swap_action_hash(
    plan: &swap::SwapPlanC,
    output: *mut u8,
    output_len: usize,
) -> u32 {
    crate::zlog("rs_swap_action_hash\x00");
    let output = std::slice::from_raw_parts_mut(output, output_len);

    if output.len() < 64 {
        return ParserError::Ok as u32;
    }

    let Ok(fvk) = c_fvk_bytes() else {
        return ParserError::InvalidFvk as u32;
    };

    let body_hash_bytes = plan.effect_hash(&fvk);

    if let Ok(body_hash_bytes) = body_hash_bytes {
        let body_hash_array = body_hash_bytes.as_array();
        let copy_len: usize = core::cmp::min(output.len(), body_hash_array.len());
        output[..copy_len].copy_from_slice(&body_hash_array[..copy_len]);
    } else {
        return ParserError::SwapPlanError as u32;
    }

    ParserError::Ok as u32
}

#[no_mangle]
/// Use to compute an address and write it back into output
/// argument.
pub unsafe extern "C" fn rs_undelegate_claim_action_hash(
    plan: &undelegate_claim::UndelegateClaimPlanC,
    output: *mut u8,
    output_len: usize,
) -> u32 {
    crate::zlog("rs_undelegate_claim_action_hash\x00");
    let output = std::slice::from_raw_parts_mut(output, output_len);

    if output.len() < 64 {
        return ParserError::Ok as u32;
    }

    let body_hash_bytes = plan.effect_hash();

    if let Ok(body_hash_bytes) = body_hash_bytes {
        let body_hash_array = body_hash_bytes.as_array();
        let copy_len: usize = core::cmp::min(output.len(), body_hash_array.len());
        output[..copy_len].copy_from_slice(&body_hash_array[..copy_len]);
    } else {
        return ParserError::UndelegateClaimPlanError as u32;
    }

    ParserError::Ok as u32
}

#[no_mangle]
/// Use to compute an address and write it back into output
/// argument.
pub unsafe extern "C" fn rs_delegator_vote_action_hash(
    plan: &delegator_vote::DelegatorVotePlanC,
    output: *mut u8,
    output_len: usize,
) -> u32 {
    crate::zlog("rs_delegator_vote_action_hash\x00");
    let output = std::slice::from_raw_parts_mut(output, output_len);

    if output.len() < 64 {
        return ParserError::Ok as u32;
    }

    let Ok(fvk) = c_fvk_bytes() else {
        return ParserError::InvalidFvk as u32;
    };

    let body_hash_bytes = plan.effect_hash(&fvk);

    if let Ok(body_hash_bytes) = body_hash_bytes {
        let body_hash_array = body_hash_bytes.as_array();
        let copy_len: usize = core::cmp::min(output.len(), body_hash_array.len());
        output[..copy_len].copy_from_slice(&body_hash_array[..copy_len]);
    } else {
        return ParserError::DelegatorVotePlanError as u32;
    }

    ParserError::Ok as u32
}

#[no_mangle]
/// Use to compute an address and write it back into output
/// argument.
pub unsafe extern "C" fn rs_generic_action_hash(
    data: &BytesC,
    action_type: u8,
    output: *mut u8,
    output_len: usize,
) -> u32 {
    crate::zlog("rs_generic_action_hash\x00");
    let output = std::slice::from_raw_parts_mut(output, output_len);

    if output.len() < 64 {
        return ParserError::Ok as u32;
    }

    let action_type = ActionPlan::from(action_type);
    let effect_hash: EffectHash;
    if let Ok(data_to_hash) = data.get_bytes() {
        match action_type {
            ActionPlan::Delegate => {
                effect_hash = EffectHash::from_proto_effecting_data(
                    "/penumbra.core.component.stake.v1.Delegate",
                    data_to_hash,
                );
            }
            ActionPlan::Undelegate => {
                effect_hash = EffectHash::from_proto_effecting_data(
                    "/penumbra.core.component.stake.v1.Undelegate",
                    data_to_hash,
                );
            }
            ActionPlan::Ics20Withdrawal => {
                effect_hash = EffectHash::from_proto_effecting_data(
                    "/penumbra.core.component.ibc.v1.Ics20Withdrawal",
                    data_to_hash,
                );
            }
            _ => {
                return ParserError::UnexpectedData as u32;
            }
        }

        let body_hash_array = effect_hash.as_bytes();
        let copy_len: usize = core::cmp::min(output.len(), body_hash_array.len());
        output[..copy_len].copy_from_slice(&body_hash_array[..copy_len]);
    }

    ParserError::Ok as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::spend_key::SpendKeyBytes;
    use crate::parser::action::ActionHash;
    use crate::parser::action::ActionsHashC;
    use crate::parser::address::AddressC;
    use crate::parser::amount::AmountC;
    use crate::parser::bytes::BytesC;
    use crate::parser::clue_plan::CluePlanC;
    use crate::parser::detection::DetectionDataPlanC;
    use crate::parser::fee::FeeC;
    use crate::parser::id::IdC;
    use crate::parser::identity_key::IdentityKeyC;
    use crate::parser::memo::MemoPlanC;
    use crate::parser::memo_plain_text::MemoPlaintextC;
    use crate::parser::note::NoteC;
    use crate::parser::penalty::PenaltyC;
    use crate::parser::swap_plaintext::SwapPlaintextC;
    use crate::parser::trading_pair::TradingPairC;
    use crate::parser::value::ValueC;

    #[test]
    fn test_transaction_plan_hash() {
        let dummy_action_hashes = ActionsHashC {
            qty: 1,
            hashes: core::array::from_fn(|_| ActionHash([0u8; 64])),
        };

        // Create dummy MemoPlanC
        let memo_key_bytes =
            hex::decode("18bd5cedd0eb952244a296c1e3fba4f417ebdcc1cfec04cb9441a394316a58bd")
                .unwrap();
        let memo_plaintext_inner_bytes = hex::decode("6ece16f387e0b932082cb0cf6823590fc287d068d6f684a36d1fb19bfd6dce8b22850f535824aeb66cb8c41309e6f5b2d58ff7b651ef4e09a09c7e48d770d190880e1827b47823a1d01f0c4b438a7b43").unwrap();
        let dummy_memo_plan = MemoPlanC {
            plaintext: MemoPlaintextC {
                return_address: AddressC {
                    inner: BytesC::from_slice(&memo_plaintext_inner_bytes),
                    alt_bech32m: BytesC::default(),
                },
                text: BytesC::default(),
            },
            key: BytesC::from_slice(&memo_key_bytes),
        };

        // Create dummy DetectionDataPlanC
        let address_inner = hex::decode("e0783360338067fc2ba548f460b3f06f33d3e756ebefa8a8c08c5e12a1e667df228df0720fb9bd963894183bc447e1c7ef591fa9625d4a66b7703eec2ec1ef543454673bb61a4f2a3d861114d6891d69").unwrap();
        let rseed1 =
            hex::decode("361218d216cfe90f77f54f045ff21b464795517c05057c595fd59e4958e39417")
                .unwrap();
        let clue_plan_1 = CluePlanC {
            address: AddressC {
                inner: BytesC::from_slice(&address_inner),
                alt_bech32m: BytesC::default(),
            },
            rseed: BytesC::from_slice(&rseed1),
            precision_bits: 3,
        };

        let rseed2 =
            hex::decode("13296da8c9dfdf969be7c7bd74e67e80977cd91635eb32038619f62c732dc46a")
                .unwrap();
        let clue_plan_2 = CluePlanC {
            address: AddressC {
                inner: BytesC::from_slice(&address_inner),
                alt_bech32m: BytesC::default(),
            },
            rseed: BytesC::from_slice(&rseed2),
            precision_bits: 2,
        };

        let mut dummy_detection_data = DetectionDataPlanC::default();
        dummy_detection_data.clue_plans[0] = clue_plan_1;
        dummy_detection_data.clue_plans[1] = clue_plan_2;

        // Create TransactionPlanC with dummy data
        let transaction_plan = TransactionPlanC {
            has_parameters: false,
            has_memo: true,
            has_detection_data: true,
            actions_hashes: dummy_action_hashes,
            parameters_hash: ParametersHash([0u8; 64]),
            memo: dummy_memo_plan,
            detection_data: dummy_detection_data,
        };

        let memo_effect_hash = transaction_plan.memo.effect_hash();
        let expected_hash = "0954149b3feec5d414a22d47ce4e69f895f52431db9fdf7adf0bb5325c2520540357b206b5a04ec8685aea0e69a93a679fcb5c220cff85ebecc3d65c6d82b4d1";
        if let Ok(memo_hash_bytes) = memo_effect_hash {
            let computed_hash = hex::encode(memo_hash_bytes.as_array());
            assert_eq!(computed_hash, expected_hash);
        } else {
            panic!("memo_effect_hash is not Ok");
        }

        let detection_effect_hash = transaction_plan.detection_data.effect_hash();
        let expected_hash = "9870b8430ea82c79e2efee478446ae45c83dce05f4b892c24295c0593e759357e1f2109f0456038858bf8084811e49712b39d4127c1911ffd816bc225071c909";
        if let Ok(detection_hash_bytes) = detection_effect_hash {
            let computed_hash = hex::encode(detection_hash_bytes.as_array());
            assert_eq!(computed_hash, expected_hash);
        } else {
            panic!("detection_effect_hash is not Ok");
        }
    }

    #[test]
    fn test_spend_action_hash() {
        // Create dummy ActionC
        let dummy_amount = AmountC {
            lo: 488666442763545928,
            hi: 0,
        };

        let asset_id_bytes =
            hex::decode("29ea9c2f3371f6a487e7e95c247041f4a356f983eb064e5d2b3bcf322ca96a10")
                .unwrap();
        let dummy_asset_id = IdC {
            inner: BytesC::from_slice(&asset_id_bytes),
        };

        let dummy_value = ValueC {
            has_amount: true,
            amount: dummy_amount,
            has_asset_id: true,
            asset_id: dummy_asset_id,
        };

        let dummy_rseed_bytes =
            hex::decode("85197c5d60cf28b5ec756a657957b310072396577956fd5cd421ca62b4a6bc09")
                .unwrap();
        let dummy_address_inner = hex::decode("890bc98e3698aa4578e419b028da5672e627c280d8b06166f4c42d5366bccf1fcf3b296cd61e8d744a21f75f2fb697183e18595d8a79008539d8fb138b405db09db65cc42d54c0e772e5d42d5f20b52f").unwrap();
        let dummy_note = NoteC {
            has_value: true,
            has_address: true,
            value: dummy_value,
            rseed: BytesC::from_slice(&dummy_rseed_bytes),
            address: AddressC {
                inner: BytesC::from_slice(&dummy_address_inner),
                alt_bech32m: BytesC::default(),
            },
        };

        let dummy_randomizer_bytes =
            hex::decode("732b53ee807140dd5672768ec1a38be09c531a0c6fc185d5f51c18f5f2261d01")
                .unwrap();
        let dummy_value_blinding_bytes =
            hex::decode("f2e2f45f0ea734d7c11321cbf20427b379cfed6f71874ff97e8bcbbfce2d3d01")
                .unwrap();
        let dummy_proof_blinding_r_bytes =
            hex::decode("73ec22fcaeccfadc720dd0350cf6af7ec274a74be832e8334613638edfd2fb10")
                .unwrap();
        let dummy_proof_blinding_s_bytes =
            hex::decode("93043bfea2094b0398f0e14bccc66a9ec335bbfd1f8e8b4c2c21428947f5e50d")
                .unwrap();
        let dummy_action = spend::SpendPlanC {
            note: dummy_note,
            position: 131414504314097,
            randomizer: BytesC::from_slice(&dummy_randomizer_bytes),
            value_blinding: BytesC::from_slice(&dummy_value_blinding_bytes),
            proof_blinding_r: BytesC::from_slice(&dummy_proof_blinding_r_bytes),
            proof_blinding_s: BytesC::from_slice(&dummy_proof_blinding_s_bytes),
        };

        let spend_key = SpendKeyBytes::from([
            0xa1, 0xff, 0xba, 0x0c, 0x37, 0x93, 0x1f, 0x0a, 0x62, 0x61, 0x37, 0x52, 0x0d, 0xa6,
            0x50, 0x63, 0x2d, 0x35, 0x85, 0x3b, 0xf5, 0x91, 0xb3, 0x6b, 0xb4, 0x28, 0x63, 0x0a,
            0x4d, 0x87, 0xc4, 0xdc,
        ]);
        let fvk = spend_key.fvk().unwrap();

        let spend_action_hash = dummy_action.effect_hash(&fvk);
        let expected_hash = "c1d1826d5b769138e323498a5d26a040e2ec5b1f5fa7ade9f96d76a88896c3a3ba3a3ae5bc081c051ef48ba46973e10767f340d379553072ffdd11a4919aef1a";
        if let Ok(spend_action_hash_bytes) = spend_action_hash {
            let computed_hash = hex::encode(spend_action_hash_bytes.as_array());
            assert_eq!(computed_hash, expected_hash);
        } else {
            panic!("spend_action_hash is not Ok");
        }
    }

    #[test]
    fn test_output_action_hash() {
        // Create dummy ActionC
        let dummy_amount = AmountC {
            lo: 535446340456032950,
            hi: 0,
        };

        let asset_id_bytes =
            hex::decode("29ea9c2f3371f6a487e7e95c247041f4a356f983eb064e5d2b3bcf322ca96a10")
                .unwrap();
        let dummy_asset_id = IdC {
            inner: BytesC::from_slice(&asset_id_bytes),
        };

        let dummy_value = ValueC {
            has_amount: true,
            amount: dummy_amount,
            has_asset_id: true,
            asset_id: dummy_asset_id,
        };

        let dummy_address_inner = hex::decode("f72c37238af64e9c8517e4cac09a43a99cee8aa4cb7e2c20419f55dd06f0884bfbfa5202b88852edda3d54273de22c4ef40edb4bc54c0c14fd0b5475d33433d0bd9793c8670795eb822b94c3cbb1a412").unwrap();
        let dummy_address = AddressC {
            inner: BytesC::from_slice(&dummy_address_inner),
            alt_bech32m: BytesC::default(),
        };

        let dummy_rseed_bytes =
            hex::decode("28fc41cb8153082b110af95a0eb013a25c4248bdc25ab2f7c7e0041258d01c42")
                .unwrap();

        let dummy_value_blinding_bytes =
            hex::decode("4c19474a9edb1933a643ae2b2648131061b95b25fb6ffeafb3e53ccacf8fe700")
                .unwrap();
        let dummy_proof_blinding_r_bytes =
            hex::decode("825b816bfb539eb34a7933f362ab7b9a3fe128074a1603a5c43afb125d44e002")
                .unwrap();
        let dummy_proof_blinding_s_bytes =
            hex::decode("86ae5038cfd758ee6520792a143ea401ef8e2afbc70f65c0b6e1d58b3492b211")
                .unwrap();

        let dummy_action = output::OutputPlanC {
            value: dummy_value,
            dest_address: dummy_address,
            rseed: BytesC::from_slice(&dummy_rseed_bytes),
            value_blinding: BytesC::from_slice(&dummy_value_blinding_bytes),
            proof_blinding_r: BytesC::from_slice(&dummy_proof_blinding_r_bytes),
            proof_blinding_s: BytesC::from_slice(&dummy_proof_blinding_s_bytes),
        };

        let spend_key = SpendKeyBytes::from([
            0xa1, 0xff, 0xba, 0x0c, 0x37, 0x93, 0x1f, 0x0a, 0x62, 0x61, 0x37, 0x52, 0x0d, 0xa6,
            0x50, 0x63, 0x2d, 0x35, 0x85, 0x3b, 0xf5, 0x91, 0xb3, 0x6b, 0xb4, 0x28, 0x63, 0x0a,
            0x4d, 0x87, 0xc4, 0xdc,
        ]);
        let fvk = spend_key.fvk().unwrap();

        let output_action_hash = dummy_action.effect_hash(&fvk, &[0u8; 32]);
        let expected_hash = "da23ad386bbe7f0f9fa6432796fe2afb08356c65363dc49d6f36dc5bd28a2d518a6e13e8365accc91022f38f66dbf31426ab3dc8dfd45749be7f428980a1ac33";
        if let Ok(output_action_hash_bytes) = output_action_hash {
            let computed_hash = hex::encode(output_action_hash_bytes.as_array());
            assert_eq!(computed_hash, expected_hash);
        } else {
            panic!("output_action_hash is not Ok");
        }
    }

    #[test]
    fn test_swap_action_hash() {
        // create trading pair dummy
        let asset_1_id_bytes =
            hex::decode("29ea9c2f3371f6a487e7e95c247041f4a356f983eb064e5d2b3bcf322ca96a10")
                .unwrap();
        let dummy_trading_pair = TradingPairC {
            has_asset_1: true,
            asset_1: BytesC::from_slice(&asset_1_id_bytes),
            has_asset_2: true,
            asset_2: BytesC::from_slice(&asset_1_id_bytes),
        };

        // Create dummy delta_1_i
        let dummy_delta_1_i = AmountC {
            lo: 271899605818601126,
            hi: 0,
        };

        // Create dummy delta_2_i
        let dummy_delta_2_i = AmountC {
            lo: 372889361770024674,
            hi: 0,
        };

        // Create dummy fee
        let fee_amount = AmountC { lo: 5, hi: 0 };
        let dummy_fee = FeeC(ValueC {
            has_amount: true,
            amount: fee_amount,
            has_asset_id: false,
            asset_id: IdC {
                inner: BytesC::default(),
            },
        });

        // create dummy swap plaintext
        let dummy_address_inner = hex::decode("111584d317e870e5881689c31a02ee002742c120a20dc4856e37058ad7fbfc7b4a59a8b92b8d3cbfc6640de587047933e1765a1e776b03840492d7824cfdd7c3d3b120d0f08b88a3d9dd06e0d8c5f8ba").unwrap();
        let dummy_address = AddressC {
            inner: BytesC::from_slice(&dummy_address_inner),
            alt_bech32m: BytesC::default(),
        };
        let dummy_rseed_bytes =
            hex::decode("492544b6e359dd4ef91cb283dd3a0714122c77908a911bf83b95030f75e1da27")
                .unwrap();
        let dummy_swap_plaintext = SwapPlaintextC {
            has_trading_pair: true,
            trading_pair: dummy_trading_pair,
            has_delta_1_i: true,
            delta_1_i: dummy_delta_1_i,
            has_delta_2_i: true,
            delta_2_i: dummy_delta_2_i,
            has_claim_fee: true,
            claim_fee: dummy_fee,
            has_claim_address: true,
            claim_address: dummy_address,
            rseed: BytesC::from_slice(&dummy_rseed_bytes),
        };

        // create dummy swap action
        let dummy_fee_blinding =
            hex::decode("62b169de7af84fa05c08a5946cb62afbf57d249634c01064b6823274a9ec5a03")
                .unwrap();
        let dummy_proof_blinding_r_bytes =
            hex::decode("26180d90520e02b2752c01284191750d4b382a755a75ef488eb5f5084e84e60c")
                .unwrap();
        let dummy_proof_blinding_s_bytes =
            hex::decode("182dc91a3f1b20a1e1dafac53b83b3e12f0ef3e08fa5320668ccd6218e02c908")
                .unwrap();
        let dummy_action = swap::SwapPlanC {
            has_swap_plaintext: true,
            swap_plaintext: dummy_swap_plaintext,
            fee_blinding: BytesC::from_slice(&dummy_fee_blinding),
            proof_blinding_r: BytesC::from_slice(&dummy_proof_blinding_r_bytes),
            proof_blinding_s: BytesC::from_slice(&dummy_proof_blinding_s_bytes),
        };

        let spend_key = SpendKeyBytes::from([
            0xa1, 0xff, 0xba, 0x0c, 0x37, 0x93, 0x1f, 0x0a, 0x62, 0x61, 0x37, 0x52, 0x0d, 0xa6,
            0x50, 0x63, 0x2d, 0x35, 0x85, 0x3b, 0xf5, 0x91, 0xb3, 0x6b, 0xb4, 0x28, 0x63, 0x0a,
            0x4d, 0x87, 0xc4, 0xdc,
        ]);
        let fvk = spend_key.fvk().unwrap();

        let swap_action_hash = dummy_action.effect_hash(&fvk);
        let expected_hash = "b648fd6fb4b3801586a0d3c6881338a9da1aef1d5def434340b32b719ba7e890d65673ec5423e99668606d11f51bafd11e0158556f7c958809e9dd2b01921d7a";
        if let Ok(swap_action_hash_bytes) = swap_action_hash {
            let computed_hash = hex::encode(swap_action_hash_bytes.as_array());
            assert_eq!(computed_hash, expected_hash);
        } else {
            panic!("output_action_hash is not Ok");
        }
    }

    #[test]
    fn test_memo_hash() {
        // Create dummy MemoPlanC
        let memo_key_bytes =
            hex::decode("d6b269dbe8d6e04bdbba2025285d956864c723c3932ba608db6fd433a194731b")
                .unwrap();
        let memo_inner_bytes = hex::decode("8d5b14d34c66c974180c1c3537b4c6167759244fc34a3fcd582f6e937a48aa27939fdb08733c64a49a59461977b6a45e5201fd087fef594b117f3e6628e1889ecc382d5d5dfc8a383fa51ff84119bc85").unwrap();
        let memo_plaintext = hex::decode("7a20383842204f736d334a6f3020204b713567204820354b5a4a35203736536251774a6e71316450306b33664152303620654257205a345720315837734c4820577420363420336c6d4e536b30495073664b3020204c20203951204b357336204c466a206571202041204c396d20204f4e2020577849202043656d333944584a20733930506350203139694368316757783132376642204f51334a32205a3820396f3020534e6e4c20505a667a69204a4639334848202073452048437666494b62532067355075675a43206877654d").unwrap();
        let dummy_memo_plan = MemoPlanC {
            plaintext: MemoPlaintextC {
                return_address: AddressC {
                    inner: BytesC::from_slice(&memo_inner_bytes),
                    alt_bech32m: BytesC::default(),
                },
                text: BytesC::from_slice(&memo_plaintext),
            },
            key: BytesC::from_slice(&memo_key_bytes),
        };

        let memo_hash = dummy_memo_plan.effect_hash();
        let expected_hash = "2f3fbb301cf857926eebf1339fc49ebff5eef78488e5e50414eeefd046f83bd40b8bd0e8cd2ec592aab1a0b83f9c800d8079d5378393f26a71bf57489b6280fc";
        if let Ok(memo_hash_bytes) = memo_hash {
            let computed_hash = hex::encode(memo_hash_bytes.as_array());
            assert_eq!(computed_hash, expected_hash);
        } else {
            panic!("output_action_hash is not Ok");
        }
    }

    #[test]
    fn test_undelegate_claim_action_hash() {
        // Create dummy ActionC
        let dummy_amount = AmountC {
            lo: 74432202791432231,
            hi: 0,
        };

        // Create dummy validator identity
        let ik_bytes =
            hex::decode("1e32c63102334a0fdfed9fdd4aa6b088824d1d42ad40109f4a56f8845dfb0e32")
                .unwrap();
        let dummy_validator_identity = IdentityKeyC {
            ik: BytesC::from_slice(&ik_bytes),
        };

        // Create dummy penalty
        let inner_bytes =
            hex::decode("00000000000000000000000000000000fecbfb15b573eab367a0f9096bb98c7f")
                .unwrap();
        let dummy_penalty = PenaltyC {
            inner: BytesC::from_slice(&inner_bytes),
        };

        let dummy_balance_blinding_bytes =
            hex::decode("02a147e3c45b43f4f0cc9d8d6e2940c6927cbb5141b6062aae8ac3ba10ac4504")
                .unwrap();
        let dummy_proof_blinding_r_bytes =
            hex::decode("a6f9bd68892e3c662cd41452dca6c196e31ce877f9e7303166e4eb25b496aa0a")
                .unwrap();
        let dummy_proof_blinding_s_bytes =
            hex::decode("84d5bd78cab02b9528a4135abcdcf46d4953ab230d1e6d4dc295eb4208b5bf0d")
                .unwrap();

        let dummy_action = undelegate_claim::UndelegateClaimPlanC {
            has_validator_identity: true,
            validator_identity: dummy_validator_identity,
            start_epoch_index: 0,
            has_penalty: true,
            penalty: dummy_penalty,
            has_unbonding_amount: true,
            unbonding_amount: dummy_amount,
            balance_blinding: BytesC::from_slice(&dummy_balance_blinding_bytes),
            proof_blinding_r: BytesC::from_slice(&dummy_proof_blinding_r_bytes),
            proof_blinding_s: BytesC::from_slice(&dummy_proof_blinding_s_bytes),
            unbonding_start_height: 25928,
        };

        let output_action_hash: Result<EffectHash, ParserError> = dummy_action.effect_hash();
        let expected_hash = "6d780b622209a23a6ac8d2895abceb8420f89b611a2a5767e146aead8aa337a6a8999f258f80a7a2c6f4c21bff674615e0ea1430c4bf86b7d1ab76565d08569e";
        if let Ok(output_action_hash_bytes) = output_action_hash {
            let computed_hash = hex::encode(output_action_hash_bytes.as_array());
            assert_eq!(computed_hash, expected_hash);
        } else {
            panic!(
                "output_action_hash is not Ok Error: {:?}",
                output_action_hash
            );
        }
    }

    #[test]
    fn test_delegator_vote_action_hash() {
        // Create dummy ActionC
        let dummy_amount = AmountC {
            lo: 881370723936900418,
            hi: 0,
        };

        let asset_id_bytes =
            hex::decode("29ea9c2f3371f6a487e7e95c247041f4a356f983eb064e5d2b3bcf322ca96a10")
                .unwrap();
        let dummy_asset_id = IdC {
            inner: BytesC::from_slice(&asset_id_bytes),
        };

        let dummy_value = ValueC {
            has_amount: true,
            amount: dummy_amount,
            has_asset_id: true,
            asset_id: dummy_asset_id,
        };

        let dummy_rseed_bytes =
            hex::decode("7c14e7434fde0abeccbc2579e58eeb65045e538b14cad708c988075b9fc0df66")
                .unwrap();
        let dummy_address_inner = hex::decode("7616f6c402371db1fa79eca16f1892132bbc1ea65e133fa67388049719f62f45c36fe666cc95ecc4444f6561a36d30fa6aad47a89032c8966f05a7cb098f9fd9ee392d0d337f3c35a33284ed4317f392").unwrap();
        let dummy_note = NoteC {
            has_value: true,
            has_address: true,
            value: dummy_value,
            rseed: BytesC::from_slice(&dummy_rseed_bytes),
            address: AddressC {
                inner: BytesC::from_slice(&dummy_address_inner),
                alt_bech32m: BytesC::default(),
            },
        };

        let dummy_unbonded_amount = AmountC {
            lo: 254692294976886837,
            hi: 0,
        };

        let dummy_randomizer_bytes =
            hex::decode("8ee3fae74bc73f0107e4f6fbb6a58be4326a0d6991af104f825b8ee4387a6b01")
                .unwrap();
        let dummy_proof_blinding_r_bytes =
            hex::decode("3ad8f590111f2259243cc440cd5aebcce1f96c719095b6d68f6111ceb1f1ae05")
                .unwrap();
        let dummy_proof_blinding_s_bytes =
            hex::decode("f59b1272a4d5ba8bea095233e51b392fe32ec7b97667aa44bf9f89321810ec10")
                .unwrap();
        let dummy_action = delegator_vote::DelegatorVotePlanC {
            proposal: 267193148,
            start_position: 20,
            has_vote: true,
            vote: 1,
            has_staked_note: true,
            staked_note: dummy_note,
            staked_note_position: 30,
            has_unbonded_amount: true,
            unbonded_amount: dummy_unbonded_amount,
            randomizer: BytesC::from_slice(&dummy_randomizer_bytes),
            proof_blinding_r: BytesC::from_slice(&dummy_proof_blinding_r_bytes),
            proof_blinding_s: BytesC::from_slice(&dummy_proof_blinding_s_bytes),
        };

        let spend_key = SpendKeyBytes::from([
            0xa1, 0xff, 0xba, 0x0c, 0x37, 0x93, 0x1f, 0x0a, 0x62, 0x61, 0x37, 0x52, 0x0d, 0xa6,
            0x50, 0x63, 0x2d, 0x35, 0x85, 0x3b, 0xf5, 0x91, 0xb3, 0x6b, 0xb4, 0x28, 0x63, 0x0a,
            0x4d, 0x87, 0xc4, 0xdc,
        ]);
        let fvk = spend_key.fvk().unwrap();

        let spend_action_hash = dummy_action.effect_hash(&fvk);
        let expected_hash = "9b886b91ecbbbe24a6fc01dd425d4954f1e28ddd9bd3c8e446e5f447313c7b3754f7644c1cd6c934fdbfd6a926c16993241ad44ed55b8cf98a0c8f3c2e9ad4ec";
        if let Ok(spend_action_hash_bytes) = spend_action_hash {
            let computed_hash = hex::encode(spend_action_hash_bytes.as_array());
            assert_eq!(computed_hash, expected_hash);
        } else {
            panic!("spend_action_hash is not Ok");
        }
    }
}
