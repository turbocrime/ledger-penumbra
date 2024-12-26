use crate::{constants::KEY_LEN, ParserError};

use crate::expand_fq::expand_ff;
use decaf377::Fq;

use super::spend_key::SpendKeyBytes;

/// Nullifier Key
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct NullifierKey(pub(crate) Fq);

impl NullifierKey {
    pub const LEN: usize = KEY_LEN;
    pub const LABEL: &'static [u8; 16] = b"Penumbra_ExpndSd";

    pub fn derive_from(spend_bytes: &SpendKeyBytes) -> Result<Self, ParserError> {
        // in Docs:
        // nk  = from_le_bytes(prf_expand("Penumbra_ExpndSd", spend_key_bytes, 1)) mod q
        let one = [1; 1];
        let nk = expand_ff(Self::LABEL, spend_bytes.key_bytes(), &one)?;
        Ok(Self(nk))
    }

    /// Convert the key to a le byte array representation
    pub fn to_bytes(self) -> [u8; Self::LEN] {
        self.0.to_bytes()
    }
}

impl TryFrom<&SpendKeyBytes> for NullifierKey {
    type Error = ParserError;

    fn try_from(value: &SpendKeyBytes) -> Result<Self, Self::Error> {
        Self::derive_from(value)
    }
}

impl TryFrom<&[u8; 32]> for NullifierKey {
    type Error = ParserError;

    fn try_from(value: &[u8; 32]) -> Result<Self, Self::Error> {
        Ok(Self(Fq::from_bytes_checked(value).unwrap()))
    }
}
