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
use self::{
    action::ActionC, action::ActionPlan, detection::DetectionDataPlan,
    detection::DetectionDataPlanC, memo::MemoPlanC,
};

use super::{ObjectList, TransactionParameters};

pub mod action;
pub mod detection;
pub mod memo;
pub mod memo_plain_text;
pub mod spend;
pub mod symmetric;

use super::tx_parameters::TransactionParametersC;
use crate::constants::ACTION_DATA_QTY;
use crate::ParserError;

// The TransactionPlan contains a declarative description of all details of
// the proposed transaction, including a plan of each action in a transparent form,
// the fee specified, the chain ID, and so on.
//
// The signing process first takes a TransactionPlan and SpendKey and
// returns the AuthorizationData, essentially a bundle of signatures over the effect hash, which
// can be computed directly from the plan data
// Describes a planned transaction. Permits clients to prepare a transaction prior submission, so that a user can review it prior to authorizing its execution.
// The TransactionPlan is a fully determined bundle binding all of a transaction's effects. The only thing it does not include is the witness data used for proving.
#[cfg_attr(test, derive(Debug))]
#[derive(Copy, PartialEq, Eq, Clone)]
pub struct TransactionPlan<'a> {
    action_plans: ObjectList<'a, ActionPlan<'a>>,
    transaction_parameters: TransactionParameters<'a>,
    detection_data: DetectionDataPlan<'a>,
    //memo: MemoPlan<'a>,
}

// impl<'a> FromBytes<'a> for TransactionPlan<'a> {
//     fn from_bytes_into(
//         input: &'a [u8],
//         out: &mut core::mem::MaybeUninit<Self>,
//     ) -> Result<&'a [u8], nom::Err<crate::ParserError>> {
//         let out = out.as_mut_ptr();

//         // Actions
//         let (rem, num_actions) = varint(input)?;
//         let action: &mut MaybeUninit<ObjectList<'a, ActionPlan<'a>>> =
//             unsafe { &mut *addr_of_mut!((*out).action_plans).cast() };
//         let rem = ObjectList::new_into_with_len(rem, action, num_actions as _)?;

//         // Transaction parameters
//         let parameters = unsafe { &mut *addr_of_mut!((*out).transaction_parameters).cast() };
//         let rem = TransactionParameters::from_bytes_into(rem, parameters)?;

//         // detection data
//         let detection = unsafe { &mut *addr_of_mut!((*out).detection_data).cast() };
//         let rem = DetectionDataPlan::from_bytes_into(rem, detection)?;

//         // Memo
//         let memo = unsafe { &mut *addr_of_mut!((*out).memo).cast() };
//         let rem = MemoPlan::from_bytes_into(rem, memo)?;

//         Ok(rem)
//     }
// }

#[repr(C)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct TransactionPlanC {
    pub actions: [ActionC; ACTION_DATA_QTY],
    pub transaction_parameters: TransactionParametersC,
    pub memo: MemoPlanC,
    pub detection_data: DetectionDataPlanC,
}

#[no_mangle]
/// Use to compute an address and write it back into output
/// argument.
pub unsafe extern "C" fn rs_compute_effect_hash() -> u32 {
    crate::zlog("rs_compute_effect_hash\x00");

    ParserError::Ok as u32
}

#[no_mangle]
/// Use to compute an address and write it back into output
/// argument.
pub unsafe extern "C" fn rs_compute_transaction_plan(
    plan: &TransactionPlanC,
    output: *mut u8,
    output_len: usize,
) -> u32 {
    crate::zlog("rs_compute_transaction_plan\x00");
    let output = std::slice::from_raw_parts_mut(output, output_len);

    if output.len() < 200 {
        return ParserError::Ok as u32;
    }

    let transaction_parameters_hash = plan.transaction_parameters.effect_hash();
    if let Ok(transaction_parameters_hash_bytes) = transaction_parameters_hash {
        let transaction_parameters_hash_array = transaction_parameters_hash_bytes.as_array();
        let copy_len: usize = core::cmp::min(output.len(), transaction_parameters_hash_array.len());
        output[..copy_len].copy_from_slice(&transaction_parameters_hash_array[..copy_len]);
    }

    if let Ok(memo_hash_bytes) = plan.memo.effect_hash() {
        let memo_hash_array = memo_hash_bytes.as_array();
        let copy_len: usize = core::cmp::min(output.len() - 64, memo_hash_array.len());
        output[68..68 + copy_len].copy_from_slice(&memo_hash_array[..copy_len]);
    }

    let detection_hash = plan.detection_data.effect_hash();
    if let Ok(detection_hash_bytes) = detection_hash {
        let detection_hash_array = detection_hash_bytes.as_array();
        let copy_len: usize = core::cmp::min(output.len() - 136, detection_hash_array.len());
        output[136..136 + copy_len].copy_from_slice(&detection_hash_array[..copy_len]);
    }

    ParserError::Ok as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::address::AddressC;
    use crate::parser::bytes::BytesC;
    use crate::parser::clue_plan::CluePlanC;
    use crate::parser::plans::action::ActionC;
    use crate::parser::plans::detection::DetectionDataPlanC;
    use crate::parser::plans::memo::MemoPlanC;
    use crate::parser::plans::memo_plain_text::MemoPlaintextC;
    use crate::parser::tx_parameters::TransactionParametersC;
    #[test]
    fn test_transaction_plan_hash() {
        // Create dummy ActionC
        let dummy_action = ActionC {
            action_type: 0, // Assuming 0 is a valid action type
            bytes: BytesC::from_slice(&[0u8; 32]),
        };

        // Create dummy TransactionParametersC
        let transaction_parameters_bytes =
            hex::decode("120d70656e756d6272612d746573741a020a00").unwrap();
        let dummy_transaction_parameters = TransactionParametersC {
            bytes: BytesC::from_slice(&transaction_parameters_bytes),
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
            actions: core::array::from_fn(|_| dummy_action.clone()),
            transaction_parameters: dummy_transaction_parameters,
            memo: dummy_memo_plan,
            detection_data: dummy_detection_data,
        };

        let transaction_parameters_effect_hash =
            transaction_plan.transaction_parameters.effect_hash();
        let expected_hash = "e2b552c4c11e0bc5df75f22945c39d2c5acb6c38582716a1dd7d87e1cfa4043b9c32b350d927a9ae39f18b45b25f638947fa82e405a3c6ca7ea91248f9fa5ab7";
        if let Ok(transaction_parameters_hash_bytes) = transaction_parameters_effect_hash {
            let computed_hash = hex::encode(transaction_parameters_hash_bytes.as_array());
            assert_eq!(computed_hash, expected_hash);
        } else {
            panic!("transaction_parameters_effect_hash is not Ok");
        }

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
}
