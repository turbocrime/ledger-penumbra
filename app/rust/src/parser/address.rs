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

// proto:

// // A Penumbra address. An address in Penumbra is a Bech32m-encoded
// // string, with the human-readable prefix (HRP) `penumbrav2t`.
// message Address {
//   // The bytes of the address. Must be represented as a series of
//   // `uint8` (i.e. values 0 through 255), with a length of 80 elements.
//   bytes inner = 1;
//
//   // Alternatively, a Bech32m-encoded string representation of the `inner`
//   // bytes.
//   //
//   // NOTE: implementations are not required to support parsing this field.
//   // Implementations should prefer to encode the bytes in all messages they
//   // produce. Implementations must not accept messages with both `inner` and
//   // `alt_bech32m` set.
//   string alt_bech32m = 2;
// }

use core::{mem::MaybeUninit, ptr::addr_of_mut};

use nom::bytes::complete::take;

use super::bytes::BytesC;
use crate::{utils::varint, FromBytes, ParserError};

// TODO! It is unclear if this address is a raw-address to which F4Jumble and bech32 encoding
// has been applied as well, if that is the case, its length is not necessarly ADDRESS_LEN
// it could be more due to bech32 hrp and checksum.

#[cfg_attr(test, derive(Debug))]
#[derive(Copy, PartialEq, Eq, Clone)]
pub struct Address<'a>(&'a [u8]);

impl<'b> FromBytes<'b> for Address<'b> {
    fn from_bytes_into(
        input: &'b [u8],
        out: &mut MaybeUninit<Self>,
    ) -> Result<&'b [u8], nom::Err<ParserError>> {
        let out = out.as_mut_ptr();

        let (input, _) = varint(input)?; // Parse field number and wire type
        let (input, len) = varint(input)?; // Parse length
                                           //
        if len as usize == 0 {
            return Err(ParserError::InvalidLength.into());
        }

        // TODO: not necessarly equal but less than
        // if len as usize != ADDRESS_LEN {
        //     return Err(ParserError::InvalidLength.into());
        // }

        let (input, bytes) = take(len as usize)(input)?;

        unsafe {
            addr_of_mut!((*out).0).write(bytes);
        }

        Ok(input)
    }
}

#[repr(C)]
#[derive(Clone, PartialEq, Default)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct AddressC {
    pub inner: BytesC,
    pub alt_bech32m: BytesC,
}

impl AddressC {
    pub unsafe fn get_address_bytes<'a>(self) -> &'a [u8] {
        let total_len = (self.inner.len + self.alt_bech32m.len) as usize;
        std::slice::from_raw_parts(self.inner.ptr, total_len)
    }

    pub unsafe fn get_inner_bytes<'a>(self) -> &'a [u8] {
        std::slice::from_raw_parts(self.inner.ptr, self.inner.len as usize)
    }

    pub unsafe fn get_alt_bech32m_bytes<'a>(self) -> &'a [u8] {
        std::slice::from_raw_parts(self.alt_bech32m.ptr, self.alt_bech32m.len as usize)
    }
}
