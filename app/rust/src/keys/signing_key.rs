use crate::{constants::KEY_LEN, ParserError};

use crate::{expand_fr::expand_ff, keys::spend_key::SpendKeyBytes};
use decaf377::Fr;
use decaf377_rdsa::{SigningKey, SpendAuth, VerificationKey};

// Documentation says that the ask in used as a SigningKey in decaf377-rdsa,
// so we define this type as the ask + ak pair, wrapped up by the SigningKey type
// from decaf377-rdsa.
// TODO: What domain do we use here? lets default it to SpendAuth for now.
#[repr(C)]
#[derive(Copy, Clone)]
/// SigningKey
pub struct Sk(SigningKey<SpendAuth>);

impl Sk {
    pub const LEN: usize = KEY_LEN;
    pub const LABEL: &'static [u8; 16] = b"Penumbra_ExpndSd";

    pub fn derive_from(spend_bytes: &SpendKeyBytes) -> Result<Self, ParserError> {
        // compute Fr field
        let ask = expand_ff(Self::LABEL, spend_bytes.key_bytes(), &[0; 1])?;
        let signing_key = SigningKey::new_from_field(ask);

        Ok(Self(signing_key))
    }

    /// Returns the 32-byte encoding of the ask component
    pub fn to_bytes(self) -> [u8; Self::LEN] {
        self.0.to_bytes()
    }

    pub fn verification_key(&self) -> VerificationKey<SpendAuth> {
        self.0.into()
    }

    pub fn signing_key(&self) -> &SigningKey<SpendAuth> {
        &self.0
    }

    pub fn randomize(&self, randomizer: &Fr) -> SigningKey<SpendAuth> {
        self.0.randomize(randomizer)
    }
}
