use core::ptr::addr_of_mut;

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
use crate::{FromBytes, ParserError};

use super::spend::SpendPlan;
use crate::parser::bytes::BytesC;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ActionType {
    Spend = 0,
}

impl TryFrom<u64> for ActionType {
    type Error = ParserError;
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Spend),
            _ => Err(ParserError::InvalidActionType),
        }
    }
}

#[repr(C)]
struct SpendVariant<'a>(ActionType, SpendPlan<'a>);

#[cfg_attr(test, derive(Debug))]
#[derive(Copy, PartialEq, Eq, Clone)]
pub enum ActionPlan<'a> {
    Spend(SpendPlan<'a>),
}

impl<'a> FromBytes<'a> for ActionPlan<'a> {
    fn from_bytes_into(
        input: &'a [u8],
        out: &mut core::mem::MaybeUninit<Self>,
    ) -> Result<&'a [u8], nom::Err<ParserError>> {
        // 1. Read the action plan type
        let (rem, action_type) = (input, 0); // TODO! read from input

        match action_type {
            0 => {
                // Spend variant
                let out = out.as_mut_ptr() as *mut SpendVariant<'a>;
                let data = unsafe { &mut *addr_of_mut!((*out).1).cast() };
                let rem = SpendPlan::from_bytes_into(rem, data)?;
                unsafe {
                    addr_of_mut!((*out).0).write(ActionType::Spend);
                }
                Ok(rem)
            }
            _ => Err(ParserError::InvalidActionType.into()),
        }
    }
}

impl<'a> ActionPlan<'a> {
    pub fn action(&self) -> ActionType {
        match self {
            Self::Spend(_) => ActionType::Spend,
        }
    }

    pub fn spend_plan(&self) -> Option<&SpendPlan<'a>> {
        match self {
            Self::Spend(info) => Some(info),
        }
    }
}

#[repr(C)]
#[derive(Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct ActionC {
    pub action_type: u8,
    pub bytes: BytesC,
}

