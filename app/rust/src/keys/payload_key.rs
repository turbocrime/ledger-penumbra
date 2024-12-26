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
use chacha20poly1305::Key;

use crate::{constants::PAYLOAD_KEY_LEN_BYTES, utils::read_fixed_bytes, FromBytes, ParserError};
/// Represents a symmetric `ChaCha20Poly1305` key.
///
/// Used for encrypting and decrypting notes, swaps, memos, and memo keys.
#[derive(Copy, Clone, PartialEq, Eq)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct PayloadKey<'a>(&'a [u8; PAYLOAD_KEY_LEN_BYTES]);

impl<'a> FromBytes<'a> for PayloadKey<'a> {
    fn from_bytes_into(
        input: &'a [u8],
        out: &mut core::mem::MaybeUninit<Self>,
    ) -> Result<&'a [u8], nom::Err<crate::ParserError>> {
        let out = out.as_mut_ptr();

        let (rem, bytes) = read_fixed_bytes::<PAYLOAD_KEY_LEN_BYTES>(input)?;

        unsafe {
            addr_of_mut!((*out).0).write(bytes);
        }

        Ok(rem)
    }
}

impl<'a> TryFrom<&'a [u8]> for PayloadKey<'a> {
    type Error = ParserError;

    fn try_from(slice: &'a [u8]) -> Result<Self, Self::Error> {
        let bytes: &[u8; PAYLOAD_KEY_LEN_BYTES] = slice
            .try_into()
            .map_err(|_| ParserError::UnexpectedBufferEnd)?;

        Ok(Self(bytes))
    }
}

impl<'a> TryFrom<PayloadKey<'a>> for Key {
    type Error = ParserError;

    fn try_from(slice: PayloadKey) -> Result<Self, Self::Error> {
        Ok(*Key::from_slice(slice.0.as_ref()))
    }
}
