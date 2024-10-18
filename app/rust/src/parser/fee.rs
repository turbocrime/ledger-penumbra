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

use core::{mem::MaybeUninit, ptr::addr_of_mut};

use crate::{FromBytes, ParserError};

use super::{Amount, AssetId};

/// Specifies fees paid by a transaction.
#[cfg_attr(test, derive(Debug))]
#[derive(Copy, PartialEq, Eq, Clone)]
pub struct Fee<'a> {
    // The amount of the token used to pay fees.
    amount: Amount,
    // If present, the asset ID of the token used to pay fees.
    // If absent, specifies the staking token implicitly.
    asset_id: Option<AssetId<'a>>,
}

impl<'a> Fee<'a> {
    pub fn new(amount: Amount, asset_id: Option<AssetId<'a>>) -> Self {
        Self { amount, asset_id }
    }
}

impl<'b> FromBytes<'b> for Fee<'b> {
    fn from_bytes_into(
        input: &'b [u8],
        out: &mut MaybeUninit<Self>,
    ) -> Result<&'b [u8], nom::Err<ParserError>> {
        // Amount
        let out = out.as_mut_ptr();
        let amount = unsafe { &mut *addr_of_mut!((*out).amount).cast() };
        let rem = Amount::from_bytes_into(input, amount)?;

        // Asset ID
        // remember it is an optional field
        let asset = unsafe { &mut *addr_of_mut!((*out).asset_id).cast() };
        let rem = AssetId::from_bytes_into(rem, asset)?;

        Ok(rem)
    }
}
