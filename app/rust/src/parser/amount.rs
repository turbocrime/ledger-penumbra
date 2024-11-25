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

use crate::ParserError;
use decaf377::{Fq, Fr};
use crate::constants::AMOUNT_LEN_BYTES;

#[derive(Clone, Debug)]
pub struct Amount {
    pub inner: u128,
}

impl Amount {
    pub const LEN: usize = AMOUNT_LEN_BYTES;

    pub fn to_le_bytes(&self) -> [u8; Self::LEN] {
        self.inner.to_le_bytes()
    }
}

impl TryFrom<AmountC> for Amount {
    type Error = ParserError;

    fn try_from(amount: AmountC) -> Result<Self, Self::Error> {
        let lo = amount.lo as u128;
        let hi = amount.hi as u128;
        // `hi` and `lo` represent the high/low order bytes respectively.
        //
        // We want to decode `hi` and `lo` into a single `u128` of the form:
        //
        //            hi: u64                          lo: u64
        // ┌───┬───┬───┬───┬───┬───┬───┬───┐ ┌───┬───┬───┬───┬───┬───┬───┬───┐
        // │   │   │   │   │   │   │   │   │ │   │   │   │   │   │   │   │   │
        // └───┴───┴───┴───┴───┴───┴───┴───┘ └───┴───┴───┴───┴───┴───┴───┴───┘
        //   15  14  13  12  11  10  9   8     7   6   5   4   3   2   1   0
        //
        // To achieve this, we shift `hi` 8 bytes to the left:
        let shifted = hi << 64;
        // and then add the lower order bytes:
        let inner = shifted + lo;

        Ok(Amount { inner })
    }    
}

impl Into<Fq> for Amount {
    fn into(self) -> Fq {
        Fq::from(self.inner)
    }
}

impl Into<Fr> for Amount {
    fn into(self) -> Fr {
        Fr::from(self.inner)
    }
}

#[repr(C)]
#[derive(Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct AmountC {
    pub lo: u64,
    pub hi: u64,
}
