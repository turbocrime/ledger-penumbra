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
use core::ptr::addr_of_mut;

use crate::{
    utils::{read_fixed_bytes, varint},
    FromBytes,
};

use crate::parser::{address::AddressC, bytes::BytesC};

use super::{Address, Precision};

#[cfg_attr(test, derive(Debug))]
#[derive(Copy, Clone)]
pub struct CluePlan<'a> {
    pub address: Address<'a>,
    pub rseed: &'a [u8; 32],
    pub precision: Precision,
}

impl<'a> FromBytes<'a> for CluePlan<'a> {
    fn from_bytes_into(
        input: &'a [u8],
        out: &mut core::mem::MaybeUninit<Self>,
    ) -> Result<&'a [u8], nom::Err<crate::ParserError>> {
        let out = out.as_mut_ptr();

        let addr = unsafe { &mut *addr_of_mut!((*out).address).cast() };
        let rem = Address::from_bytes_into(input, addr)?;

        let (rem, rseed) = read_fixed_bytes::<32>(rem)?;

        let (rem, precision) = varint(rem)?;

        let precision = Precision::try_from(precision)?;

        unsafe {
            addr_of_mut!((*out).rseed).write(rseed);
            addr_of_mut!((*out).precision).write(precision);
        }

        Ok(rem)
    }
}

#[repr(C)]
#[derive(Clone, Default)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct CluePlanC {
    pub address: AddressC,
    pub rseed: BytesC,
    pub precision_bits: u64,
}
