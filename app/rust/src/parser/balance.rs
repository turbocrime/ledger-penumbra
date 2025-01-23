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
    commitment::Commitment,
    value::{Imbalance, Sign, Value},
};
use crate::ParserError;
use decaf377::Fr;

// Only ten imbalances are supported for now
const IMBALANCES_SIZE: usize = 10;

#[derive(Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct Balance {
    pub imbalances: [Option<Imbalance>; IMBALANCES_SIZE],
}

impl Balance {
    pub fn new() -> Self {
        Balance {
            imbalances: [const { None }; IMBALANCES_SIZE],
        }
    }

    pub fn insert(&mut self, imbalance: Imbalance) -> Result<(), ParserError> {
        for slot in &mut self.imbalances {
            if slot.is_none() {
                *slot = Some(imbalance);
                return Ok(());
            }
        }
        Err(ParserError::InvalidLength)
    }

    pub fn commit(&self, blinding_factor: Fr) -> Result<Commitment, ParserError> {
        if !self.has_valid_imbalance() {
            return Err(ParserError::InvalidLength);
        }

        let mut commitment = decaf377::Element::IDENTITY;

        for imbalance in self.imbalances.iter().flatten() {
            let g_v = imbalance.value.asset_id.value_generator();
            let amount_fr: Fr = Into::into(imbalance.value.amount);

            if amount_fr.ne(&Fr::ZERO) {
                match imbalance.sign {
                    Sign::Required => {
                        commitment -= g_v * amount_fr;
                    }
                    Sign::Provided => {
                        commitment += g_v * amount_fr;
                    }
                }
            }
        }

        let value_blinding_generator = Commitment::value_blinding_generator();
        commitment += blinding_factor * value_blinding_generator;

        Ok(commitment.into())
    }

    fn has_valid_imbalance(&self) -> bool {
        self.imbalances.iter().any(|slot| slot.is_some())
    }

    pub fn add(&self, rhs: &Value, sign: Sign) -> Result<Balance, ParserError> {
        let mut new_balance = self.clone();

        for existing_imbalance in &mut new_balance.imbalances.iter_mut().flatten() {
            if existing_imbalance.value.asset_id == rhs.asset_id
                && existing_imbalance.sign == sign
            {
                existing_imbalance.value.amount = existing_imbalance.value.amount + rhs.amount;
                return Ok(new_balance);
            }  
        }

        new_balance.insert(Imbalance {
            value: rhs.clone(),
            sign,
        })?;

        Ok(new_balance)
    }
}

impl Default for Balance {
    fn default() -> Self {
        Self::new()
    }
}
