use crate::{utils::prf::expand, ParserError};

use super::{nk::NullifierKey, spend_key::SpendKeyBytes};

/// Allows viewing outgoing notes, i.e., notes sent from the spending key this
/// key is derived from.
#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct Ovk(pub(crate) [u8; Self::LEN]);

impl Ovk {
    pub const LEN: usize = 32;
    pub const LABEL: &'static [u8; 16] = b"Penumbra_DeriOVK";

    // Docs described it is derived as:
    // ovk  = prf_expand(b"Penumbra_DeriOVK", to_le_bytes(nk), decaf377_encode(ak))[0..32]
    pub fn derive_from(spend_bytes: &SpendKeyBytes) -> Result<Self, ParserError> {
        let ask = spend_bytes.signing_key()?;
        let nk = NullifierKey::derive_from(spend_bytes)?;

        let nk_bytes = nk.to_bytes();
        // now, the spend verification key (ak) is encoded using the function `decaf377_encode(ak)`
        // to produce its canonical encoding.
        let ak = ask.verification_key();

        // https://github.com/penumbra-zone/penumbra/blob/main/crates/core/keys/src/keys/fvk.rs#L79
        let dk_bytes = expand(Self::LABEL, &nk_bytes, ak.as_ref())?;

        let mut bytes = [0u8; Self::LEN];
        bytes.copy_from_slice(&dk_bytes[0..Self::LEN]);

        Ok(Self(bytes))
    }

    pub fn to_bytes(&self) -> [u8; Self::LEN] {
        self.0
    }
}

impl TryFrom<&SpendKeyBytes> for Ovk {
    type Error = ParserError;

    fn try_from(value: &SpendKeyBytes) -> Result<Self, Self::Error> {
        Self::derive_from(value)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const SPEND_KEY: &str = "ff726c71bcec76abc6a88cba71df655b28de6580edbd33c7415fdfded2e422e7";
    const EXPECTED_OVK: &str = "7e2a61790b91bd896bf9bd14a3723934011b166413128a4aad02675897397902";

    #[test]
    fn test_derive_ovk() {
        let key_bytes = hex::decode(SPEND_KEY).unwrap();
        let key_bytes: [u8; 32] = key_bytes.as_slice().try_into().unwrap();
        let spk = SpendKeyBytes::from(key_bytes);

        let ovk = Ovk::derive_from(&spk).unwrap();
        let ovk = hex::encode(ovk.0);

        assert_eq!(ovk, EXPECTED_OVK);
    }
}
