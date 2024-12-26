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

use crate::parser::bytes::BytesC;
use crate::parser::id::{Id, IdC};
use crate::ParserError;

#[derive(Clone, Debug)]
pub struct TradingPair {
    pub(crate) asset_1: Id,
    pub(crate) asset_2: Id,
}

#[repr(C)]
#[derive(Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct TradingPairC {
    pub has_asset_1: bool,
    pub asset_1: BytesC,
    pub has_asset_2: bool,
    pub asset_2: BytesC,
}

impl TryFrom<TradingPairC> for TradingPair {
    type Error = ParserError;

    fn try_from(value: TradingPairC) -> Result<Self, Self::Error> {
        let id_1 = IdC {
            inner: value.asset_1,
        };
        let id_2 = IdC {
            inner: value.asset_2,
        };

        Ok(Self {
            asset_1: Id::try_from(id_1)?,
            asset_2: Id::try_from(id_2)?,
        })
    }
}

impl TradingPair {
    pub const PROTO_LEN: usize = 2 * Id::LEN + 10;
    pub const PROTO_PREFIX_ASSET_1: [u8; 6] = [0x0a, 0x48, 0x0a, 0x22, 0x0a, 0x20];
    pub const PROTO_PREFIX_ASSET_2: [u8; 4] = [0x12, 0x22, 0x0a, 0x20];

    pub fn asset_1(&self) -> &Id {
        &self.asset_1
    }

    pub fn asset_2(&self) -> &Id {
        &self.asset_2
    }

    pub fn to_bytes(&self) -> Result<[u8; 64], ParserError> {
        let mut bytes = [0u8; 64];
        let asset_1_bytes = self.asset_1.to_bytes();
        let asset_2_bytes = self.asset_2.to_bytes();

        bytes[..32].copy_from_slice(&asset_1_bytes);
        bytes[32..].copy_from_slice(&asset_2_bytes);

        Ok(bytes)
    }

    pub fn to_proto(&self) -> Result<[u8; Self::PROTO_LEN], ParserError> {
        let mut proto = [0u8; Self::PROTO_LEN];

        proto[0..6].copy_from_slice(&Self::PROTO_PREFIX_ASSET_1);
        proto[6..38].copy_from_slice(&self.asset_1.to_bytes());
        proto[38..42].copy_from_slice(&Self::PROTO_PREFIX_ASSET_2);
        proto[42..74].copy_from_slice(&self.asset_2.to_bytes());

        Ok(proto)
    }
}
