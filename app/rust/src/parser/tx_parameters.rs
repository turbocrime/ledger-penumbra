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
use crate::{
    effect_hash::{create_personalized_state, EffectHash},
    utils::{read_string, varint},
    FromBytes, ParserError,
};

use super::bytes::BytesC;
use super::Fee;

/// The parameters determining if a transaction should be accepted by the chain.
#[derive(Copy, PartialEq, Eq, Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct TransactionParameters<'a> {
    expiry_height: u64,
    chain_id: &'a str,
    fee: Fee<'a>,
    bytes: &'a [u8],
}

#[repr(C)]
#[derive(Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct TransactionParametersC {
    pub bytes: BytesC,
}

impl<'a> TransactionParameters<'a> {
    pub fn new(expiry_height: u64, chain_id: &'a str, fee: Fee<'a>, bytes: &'a [u8]) -> Self {
        Self {
            expiry_height,
            chain_id,
            fee,
            bytes,
        }
    }
}

impl<'a> FromBytes<'a> for TransactionParameters<'a> {
    fn from_bytes_into(
        input: &'a [u8],
        out: &mut core::mem::MaybeUninit<Self>,
    ) -> Result<&'a [u8], nom::Err<crate::ParserError>> {
        let out = out.as_mut_ptr();

        let (rem, expiry_height) = varint(input)?;
        let (rem, chain_id) = read_string(rem)?;

        let fee = unsafe { &mut *addr_of_mut!((*out).fee).cast() };
        let rem = Fee::from_bytes_into(rem, fee)?;

        unsafe {
            addr_of_mut!((*out).expiry_height).write(expiry_height);
            addr_of_mut!((*out).chain_id).write(chain_id);
        }

        Ok(rem)
    }
}

impl TransactionParametersC {
    pub fn effect_hash(&self) -> Result<EffectHash, ParserError> {
        let mut state =
            create_personalized_state("/penumbra.core.transaction.v1.TransactionParameters");
        let bytes = unsafe { core::slice::from_raw_parts(self.bytes.ptr, self.bytes.len as usize) };
        state.update(bytes);
        let hash = state.finalize();
        Ok(EffectHash(*hash.as_array()))
    }
}
