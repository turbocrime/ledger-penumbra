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
    detection::DetectionDataPlanC,
    memo::MemoPlanC,
    action::ActionsHashC,
    action::ActionPlan,
};

use crate::keys::spend_key::SpendKeyBytes;
use crate::parser::effect_hash::EffectHash;
use crate::parser::bytes::BytesC;
use crate::ParserError;
use crate::constants::EFFECT_HASH_LEN;
use crate::parser::parameters::ParametersHash;

pub mod output;
pub mod spend;

#[repr(C)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct TransactionPlanC {
    pub actions_hashes: ActionsHashC,
    pub parameters_hash: ParametersHash,
    pub memo: MemoPlanC,
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
        effect_hash = EffectHash::from_proto_effecting_data("/penumbra.core.transaction.v1.TransactionParameters", data_to_hash);

        let body_hash_array = effect_hash.as_bytes();
        let copy_len: usize = core::cmp::min(output.len(), body_hash_array.len());
        output[..copy_len].copy_from_slice(&body_hash_array[..copy_len]);
    }

    ParserError::Ok as u32
}

#[no_mangle]
/// Use to compute an address and write it back into output
/// argument.
pub unsafe extern "C" fn rs_spend_action_hash(
    sk: &SpendKeyBytes,
    plan: &spend::SpendPlanC,
    output: *mut u8,
    output_len: usize,
) -> u32 {
    crate::zlog("rs_spend_action_hash\x00");
    let output = std::slice::from_raw_parts_mut(output, output_len);

    if output.len() < 64 {
        return ParserError::Ok as u32;
    }

    let fvk = sk.fvk().unwrap();
    let body_hash_bytes = plan.effect_hash(&fvk);

    if let Ok(body_hash_bytes) = body_hash_bytes {
        let body_hash_array = body_hash_bytes.as_array();
        let copy_len: usize = core::cmp::min(output.len(), body_hash_array.len());
        output[..copy_len].copy_from_slice(&body_hash_array[..copy_len]);
    }

    ParserError::Ok as u32
}

#[no_mangle]
/// Use to compute an address and write it back into output
/// argument.
pub unsafe extern "C" fn rs_output_action_hash(
    sk: &SpendKeyBytes,
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

    let fvk: crate::keys::FullViewingKey = sk.fvk().unwrap();
    let memo_key_bytes = match memo_key.get_bytes() {
        Ok(bytes) => bytes,
        Err(_) => &[0u8; 32],
    };

    let body_hash_bytes = plan.effect_hash(&fvk, &memo_key_bytes);

    if let Ok(body_hash_bytes) = body_hash_bytes {
        let body_hash_array = body_hash_bytes.as_array();
        let copy_len: usize = core::cmp::min(output.len(), body_hash_array.len());
        output[..copy_len].copy_from_slice(&body_hash_array[..copy_len]);
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
                effect_hash = EffectHash::from_proto_effecting_data("/penumbra.core.component.stake.v1.Delegate", data_to_hash);
            }
            ActionPlan::Undelegate => {
                effect_hash = EffectHash::from_proto_effecting_data("/penumbra.core.component.stake.v1.Undelegate", data_to_hash);
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
    use crate::parser::address::AddressC;
    use crate::parser::bytes::BytesC;
    use crate::parser::clue_plan::CluePlanC;
    use crate::parser::note::NoteC;
    use crate::parser::action::ActionsHashC;
    use crate::parser::amount::AmountC;
    use crate::parser::detection::DetectionDataPlanC;
    use crate::parser::id::IdC;
    use crate::parser::memo::MemoPlanC;
    use crate::parser::memo_plain_text::MemoPlaintextC;
    use crate::parser::value::ValueC;
    use crate::parser::action::ActionHash;
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
            amount: dummy_amount,
            asset_id: dummy_asset_id,
        };

        let dummy_rseed_bytes =
            hex::decode("85197c5d60cf28b5ec756a657957b310072396577956fd5cd421ca62b4a6bc09")
                .unwrap();
        let dummy_address_inner = hex::decode("890bc98e3698aa4578e419b028da5672e627c280d8b06166f4c42d5366bccf1fcf3b296cd61e8d744a21f75f2fb697183e18595d8a79008539d8fb138b405db09db65cc42d54c0e772e5d42d5f20b52f").unwrap();
        let dummy_note = NoteC {
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
            amount: dummy_amount,
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
}
