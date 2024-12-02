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

use decaf377::Fq;

use crate::keys::dk::Diversifier;
use crate::keys::{ka, ClueKey};
use crate::{keys::dk::DiversifierKey, ParserError};

pub mod address_index;
pub mod address_view;

use crate::constants::ADDRESS_LEN;

#[derive(Clone, Copy, PartialEq, Debug)]
// pub struct Address([u8; Address::LEN]);
/// A valid payment address.
pub struct Address {
    d: Diversifier,
    /// cached copy of the diversified base
    g_d: decaf377::Element,
    /// extra invariant: the bytes in pk_d should be the canonical encoding of an
    /// s value (whether or not it is a valid decaf377 encoding)
    /// this ensures we can use a PaymentAddress to form a note commitment,
    /// which involves hashing s as a field element.
    pk_d: ka::Public,
    tsk: Fq,
    ck_d: ClueKey,
}

// Steps for deriving an address from spendykeyBytes:
// 1. generate a Diversifier for address index i (Di).
// 2. generate a diversifier basepoint(Bd).
// 3. generate a detection key(Dtk).
// 4. compute a transmission key(Pkd).
// 5. compute the clue key(Ckd).
// 6. concatenate (Di, Pkd, Ckd) to form a raw binary encoding of the payment address(80-bytes).
// 7. apply the F4Jumble function to the raw binary encoding.
// 8. Encode the jumbled string with Bech32m, using the prefix "penumbra" for mainnet addresses or
//    "penumbra*tnXYZ*" for testnet addresses, where "XYZ" is the testnet number.
impl Address {
    pub const LEN: usize = ADDRESS_LEN;
    // Max Length=length(HRP)+1+(8×Data_Size(in bits)/5)+6
    // Max Length=8+1+128+6=143
    pub const MAX_ENC_LEN: usize = 150;

    /// Number of bits in the address short form divided by the number of bits per Bech32m character
    pub const ADDRESS_NUM_CHARS_SHORT_FORM: usize = 24;

    /// Use to fill buffer with an raw address(before F4Jumble and bech32 encoding)
    /// for index `idx` and `spend_key`
    /// Returns Ok on success
    pub fn new(_spend_key: &[u8; Self::LEN], _idx: u16) -> Result<Self, ParserError> {
        todo!()
    }

    /// Constructs a payment address from its components.
    ///
    /// Returns `None` if the bytes in pk_d are a non-canonical representation
    /// of an [`Fq`] `s` value.
    pub fn from_components(
        d: Diversifier,
        pk_d: ka::Public,
        ck_d: ClueKey,
    ) -> Result<Self, ParserError> {
        // XXX ugly -- better way to get our hands on the s value?
        // add to decaf377::Encoding? there's compress_to_field already...
        // if let Ok(tsk) = Fq::deserialize_compressed(&pk_d.0[..]) {
        Fq::from_bytes_checked(&pk_d.0)
            .map(|tsk| Self {
                d,
                g_d: d.diversified_generator(),
                pk_d,
                ck_d,
                tsk,
            })
            .map_err(|_| ParserError::InvalidAddress)
    }

    /// Returns a reference to the diversified base.
    ///
    /// This method computes the diversified base if it has not been computed yet. This value is
    /// cached after it has been computed once.
    pub fn diversified_generator(&self) -> &decaf377::Element {
        &self.g_d
    }

    pub fn diversifier(&self) -> &Diversifier {
        &self.d
    }

    pub fn transmission_key(&self) -> &ka::Public {
        &self.pk_d
    }

    pub fn transmission_key_s(&self) -> &Fq {
        &self.tsk
    }

    pub fn clue_key(&self) -> &ClueKey {
        &self.ck_d
    }

    pub fn to_bytes(&self) -> Result<[u8; Self::LEN], ParserError> {
        let mut bytes = [0; Self::LEN];
        bytes[0..16].copy_from_slice(self.diversifier().as_ref());
        bytes[16..48].copy_from_slice(&self.transmission_key().0);
        bytes[48..80].copy_from_slice(&self.clue_key().0);
        f4jumble::f4jumble_mut(&mut bytes).map_err(|_| ParserError::InvalidLength)?;
        Ok(bytes)
    }

    
}

impl TryFrom<&[u8]> for Address {
    type Error = ParserError;

    fn try_from(jumbled_bytes: &[u8]) -> Result<Self, Self::Error> {
        if jumbled_bytes.len() != ADDRESS_LEN {
            return Err(ParserError::InvalidLength);
        }
        let mut unjumbled_bytes = [0u8; 80];
        unjumbled_bytes.copy_from_slice(jumbled_bytes);

        f4jumble::f4jumble_inv_mut(&mut unjumbled_bytes).map_err(|_| ParserError::InvalidAddress)?;

        let diversifier_bytes = &unjumbled_bytes[0..16];

        let pk_d_bytes = &unjumbled_bytes[16..48];

        let clue_key_bytes = &unjumbled_bytes[48..80];

        let diversifier = Diversifier(diversifier_bytes.try_into().expect("can form diversifier bytes"));

        Address::from_components(
            diversifier,
            ka::Public(pk_d_bytes.try_into().expect("can form pk_d bytes")),
            ClueKey(clue_key_bytes.try_into().expect("can form clue_key bytes")),
        )
        .map_err(|_| ParserError::InvalidAddress)
    }
}

#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct AddressIndex {
    pub account: u32,
    pub randomizer: [u8; Self::RAND_LEN],
}

impl AddressIndex {
    pub const RAND_LEN: usize = 12;

    pub fn to_bytes(self) -> [u8; 16] {
        let mut bytes = [0; DiversifierKey::LEN];
        bytes[0..4].copy_from_slice(&self.account.to_le_bytes());
        bytes[4..16].copy_from_slice(&self.randomizer);
        bytes
    }

    pub fn is_ephemeral(&self) -> bool {
        self.randomizer != [0; 12]
    }

    pub fn new(account: u32) -> Self {
        AddressIndex::from(account)
    }
}

impl From<u32> for AddressIndex {
    fn from(x: u32) -> Self {
        Self {
            account: x,
            randomizer: [0; 12],
        }
    }
}

impl From<AddressIndex> for u128 {
    fn from(x: AddressIndex) -> Self {
        u128::from_le_bytes(x.to_bytes())
    }
}

impl TryFrom<AddressIndex> for u64 {
    type Error = ParserError;
    fn try_from(address_index: AddressIndex) -> Result<Self, Self::Error> {
        let mut bytes = [0; 8];
        bytes[0..4].copy_from_slice(&address_index.account.to_le_bytes());
        bytes[5..8].copy_from_slice(address_index.randomizer.as_slice());

        Ok(u64::from_le_bytes(bytes))
    }
}

impl TryFrom<&[u8]> for AddressIndex {
    type Error = ParserError;

    fn try_from(slice: &[u8]) -> Result<AddressIndex, Self::Error> {
        if slice.len() != DiversifierKey::LEN {
            return Err(ParserError::InvalidLength);
        }

        Ok(AddressIndex {
            account: u32::from_le_bytes(slice[0..4].try_into().expect("can form 4 byte array")),
            randomizer: slice[4..16].try_into().expect("can form 12 byte array"),
        })
    }
}
