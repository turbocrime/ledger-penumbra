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

#[cfg(any(feature = "derive-debug", test))]
use core::fmt;

#[repr(C)]
#[derive(Clone, PartialEq)]
pub struct BytesC {
    pub ptr: *const u8,
    pub len: u16,
}

impl BytesC {
    pub fn into_array<const L: usize>(&self) -> Result<[u8; L], ParserError> {
        let slice: &[u8] = self.into();
        slice.try_into().map_err(|_| ParserError::InvalidLength)
    }
}

impl From<&BytesC> for &[u8] {
    fn from(value: &BytesC) -> Self {
        unsafe {
            if value.ptr.is_null() {
                &[]
            } else {
                std::slice::from_raw_parts(value.ptr, value.len as usize)
            }
        }
    }
}

#[cfg(any(feature = "derive-debug", test))]
impl fmt::Debug for BytesC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BytesC")
            .field("ptr", &self.ptr)
            .field("len", &self.len)
            .finish()
    }
}

impl BytesC {
    pub fn get_bytes(&self) -> Result<&[u8], ParserError> {
        if self.ptr.is_null() || self.len == 0 {
            Err(ParserError::UnexpectedData)
        } else {
            unsafe { Ok(std::slice::from_raw_parts(self.ptr, self.len as usize)) }
        }
    }
}

impl Default for BytesC {
    fn default() -> Self {
        BytesC {
            ptr: std::ptr::null(),
            len: 0,
        }
    }
}

impl BytesC {
    pub fn from_slice(slice: &[u8]) -> Self {
        BytesC {
            ptr: slice.as_ptr(),
            len: slice.len() as u16,
        }
    }
}
