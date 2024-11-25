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

use super::bytes::BytesC;
use crate::ParserError;
#[repr(C)]
#[derive(Clone, PartialEq, Default)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct AddressC {
    pub inner: BytesC,
    pub alt_bech32m: BytesC,
}

impl AddressC {
    pub unsafe fn get_address_bytes<'a>(self) -> Result<&'a [u8], ParserError> {
        let total_len = (self.inner.len + self.alt_bech32m.len) as usize;
        Ok(std::slice::from_raw_parts(self.inner.ptr, total_len))
    }

    pub unsafe fn get_inner_bytes<'a>(self) -> Result<&'a [u8], ParserError> {
        Ok(std::slice::from_raw_parts(self.inner.ptr, self.inner.len as usize))
    }

    pub unsafe fn get_alt_bech32m_bytes<'a>(self) -> Result<&'a [u8], ParserError> {
        Ok(std::slice::from_raw_parts(self.alt_bech32m.ptr, self.alt_bech32m.len as usize))
    }
}
