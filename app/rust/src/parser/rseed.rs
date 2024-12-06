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

use crate::constants::RSEED_LEN_BYTES;
use crate::keys::ka;
use crate::parser::bytes::BytesC;
use crate::{utils::prf, ParserError};
use decaf377::{Fq, Fr};
#[derive(Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct Rseed(pub [u8; 32]);

impl Rseed {
    pub const LEN: usize = RSEED_LEN_BYTES;

    /// Derive the ephemeral secret key from the rseed.
    pub fn derive_esk(&self) -> Result<ka::Secret, ParserError> {
        let hash_result = prf::expand(b"Penumbra_DeriEsk", &self.0, &[4u8])?;
        Ok(ka::Secret::new_from_field(Fr::from_le_bytes_mod_order(
            &hash_result,
        )))
    }

    /// Derive note commitment randomness from the rseed.
    pub fn derive_note_blinding(&self) -> Result<Fq, ParserError> {
        let hash_result = prf::expand(b"Penumbra_DeriRcm", &self.0, &[5u8])?;
        Ok(Fq::from_le_bytes_mod_order(&hash_result))
    }

    pub fn to_bytes(&self) -> Result<[u8; Self::LEN], ParserError> {
        let mut bytes = [0; Self::LEN];
        bytes.copy_from_slice(&self.0);
        Ok(bytes)
    }
}

impl TryFrom<BytesC> for Rseed {
    type Error = ParserError;

    fn try_from(value: BytesC) -> Result<Self, Self::Error> {
        assert_eq!(value.len, 32, "Invalid rseed length");
        let mut rseed = [0u8; 32];
        rseed.copy_from_slice(value.get_bytes()?);
        Ok(Rseed(rseed))
    }
}
