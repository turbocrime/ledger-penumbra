/*******************************************************************************
a   (c) 2024 Zondax GmbH
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
    amount::{Amount, AmountC},
    balance::Balance,
    trading_pair::TradingPair,
    value::Value,
    value::{Imbalance, Sign},
    ParserError,
};

pub struct Reserves {
    pub r1: Amount,
    pub r2: Amount,
}

#[repr(C)]
#[derive(Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct ReservesC {
    pub has_r1: bool,
    pub r1: AmountC,
    pub has_r2: bool,
    pub r2: AmountC,
}

impl Reserves {
    /// Augment `self` with type information to get a typed `Balance`.
    pub fn balance(&self, pair: &TradingPair) -> Result<Balance, ParserError> {
        let mut balance = Balance::new();

        let (amount_1, asset_1) = (self.r1, pair.asset_1.clone());
        let (amount_2, asset_2) = (self.r2, pair.asset_2.clone());

        if asset_1 != asset_2 {
            balance.insert(Imbalance {
                value: Value {
                    amount: amount_1,
                    asset_id: asset_1,
                },
                sign: Sign::Provided,
            })?;

            balance.insert(Imbalance {
                value: Value {
                    amount: amount_2,
                    asset_id: asset_2,
                },
                sign: Sign::Provided,
            })?;
        } else {
            let total_amount = amount_1 + amount_2;
            balance.insert(Imbalance {
                value: Value {
                    amount: total_amount,
                    asset_id: asset_1,
                },
                sign: Sign::Provided,
            })?;
        }

        Ok(balance)
    }
}

impl TryFrom<ReservesC> for Reserves {
    type Error = ParserError;
    fn try_from(value: ReservesC) -> Result<Self, Self::Error> {
        let r1 = value.r1.try_into()?;
        let r2 = value.r2.try_into()?;
        Ok(Reserves { r1, r2 })
    }
}
