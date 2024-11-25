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

use crate::constants::{MAX_TEXT_LEN, MEMO_LEN_BYTES};
use crate::parser::{address::AddressC, bytes::BytesC};

#[repr(C)]
#[derive(Clone, PartialEq, Default)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct MemoPlaintextC {
    pub return_address: AddressC,
    pub text: BytesC,
}

impl MemoPlaintextC {
    pub fn get_memo_plaintext_bytes(&self) -> Option<&[u8]> {
        let return_address_len = self.return_address.inner.len as usize;
        let alt_bech32m_len = self.return_address.alt_bech32m.len as usize;
        let text_len = self.text.len as usize;
        let total_len = return_address_len + alt_bech32m_len + text_len;

        if total_len > MEMO_LEN_BYTES || text_len > MAX_TEXT_LEN {
            return None;
        }

        unsafe {
            Some(std::slice::from_raw_parts(
                self.return_address.inner.ptr,
                total_len,
            ))
        }
    }
}
