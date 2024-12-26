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
use crate::constants::DETECTION_DATA_QTY;
use crate::parser::clue_plan::CluePlanC;
use crate::parser::effect_hash::{create_personalized_state, EffectHash};
use crate::ParserError;

#[repr(C)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct DetectionDataPlanC {
    pub clue_plans: [CluePlanC; DETECTION_DATA_QTY],
}

impl Default for DetectionDataPlanC {
    fn default() -> Self {
        DetectionDataPlanC {
            clue_plans: core::array::from_fn(|_| CluePlanC::default()),
        }
    }
}

impl DetectionDataPlanC {
    pub fn effect_hash(&self) -> Result<EffectHash, ParserError> {
        if self.is_empty() {
            return Ok(EffectHash::default());
        }

        let mut state = create_personalized_state("/penumbra.core.transaction.v1.DetectionData");
        let proto_header = [0x22, 0x46, 0x0a, 0x44];

        for clue_plan in self.clue_plans.iter() {
            if clue_plan.address.inner.ptr.is_null() || clue_plan.rseed.ptr.is_null() {
                continue;
            }

            let address_bytes = unsafe {
                core::slice::from_raw_parts(
                    clue_plan.address.inner.ptr,
                    clue_plan.address.inner.len as usize,
                )
            };

            let address =
                Address::try_from(address_bytes).map_err(|_| ParserError::InvalidAddress)?;
            let mut expanded_clue_key = address.clue_key().expand_infallible();

            let rseed_array = unsafe {
                let rseed_slice = core::slice::from_raw_parts(clue_plan.rseed.ptr, 32);
                let mut array = [0u8; 32];
                array.copy_from_slice(rseed_slice);
                array
            };

            let precision_bits = clue_plan.precision_bits as u8;
            let clue = expanded_clue_key
                .create_clue_deterministic(precision_bits, rseed_array)
                .map_err(|_| ParserError::ClueCreationFailed)?;

            state.update(&proto_header);
            unsafe {
                let clue_bytes = core::slice::from_raw_parts(clue.0.as_ptr(), 68);
                state.update(clue_bytes);
            }
        }

        let hash = state.finalize();
        Ok(EffectHash(*hash.as_array()))
    }

    pub fn is_empty(&self) -> bool {
        self.clue_plans
            .iter()
            .all(|clue_plan| clue_plan.address.inner.ptr.is_null() || clue_plan.rseed.ptr.is_null())
    }
}
