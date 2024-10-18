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

use crate::{constants::MAX_PRECISION, ParserError};

/// Represents the precision governing the false positive rate of detection.
///
/// This is usually measured in bits, where a precision of `n` bits yields false
/// positives with a rate of `2^-n`.
///
/// This type implements `TryFrom` for `u8`, `u32`, `u64`, and `i32`, which has the behavior of considering
/// the value as a number of bits, and converting if this number isn't too large.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct Precision(u8);

impl Precision {
    pub const MAX: Precision = Precision(MAX_PRECISION);

    pub fn new(precision_bits: u8) -> Result<Self, ParserError> {
        if precision_bits > MAX_PRECISION {
            return Err(ParserError::PrecisionTooLarge);
        }
        Ok(Self(precision_bits))
    }

    pub fn bits(&self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for Precision {
    type Error = ParserError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<u32> for Precision {
    type Error = ParserError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        u8::try_from(value)
            .map_err(|_| ParserError::PrecisionTooLarge)?
            .try_into()
    }
}

impl TryFrom<u64> for Precision {
    type Error = ParserError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        u8::try_from(value)
            .map_err(|_| ParserError::PrecisionTooLarge)?
            .try_into()
    }
}

impl TryFrom<i32> for Precision {
    type Error = ParserError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        u8::try_from(value)
            .map_err(|_| ParserError::PrecisionTooLarge)?
            .try_into()
    }
}
