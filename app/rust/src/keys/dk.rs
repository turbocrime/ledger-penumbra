use crate::ParserError;

use crate::address::AddressIndex;
use crate::utils::prf;

use aes::cipher::{generic_array::GenericArray, BlockDecrypt, BlockEncrypt, KeyInit};
use aes::Aes128;
use decaf377::{Element, Fq};

use super::spend_key::SpendKeyBytes;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Diversifier(pub(crate) [u8; Diversifier::LEN]);

impl Diversifier {
    pub const LEN: usize = 16;
    pub const LABEL: &'static [u8; 16] = b"Penumbra_Divrsfy";

    /// Generate the diversified basepoint associated to this diversifier.
    pub fn diversified_generator(&self) -> Element {
        crate::zlog("Diversifier::diversified_generator\x00");
        // let hash = blake2b_simd::Params::new()
        //     .personal(Self::LABEL)
        //     .hash(&self.0);
        // Element::encode_to_curve(&Fq::from_le_bytes_mod_order(hash.as_bytes()))
        let hash = prf::expand(Self::LABEL, &[], &self.0).expect("can expand");
        Element::encode_to_curve(&Fq::from_le_bytes_mod_order(&hash))
    }
}

impl AsRef<[u8; Diversifier::LEN]> for Diversifier {
    fn as_ref(&self) -> &[u8; Diversifier::LEN] {
        &self.0
    }
}

impl TryFrom<&[u8]> for Diversifier {
    type Error = ParserError;

    fn try_from(slice: &[u8]) -> Result<Diversifier, Self::Error> {
        if slice.len() != Diversifier::LEN {
            return Err(ParserError::InvalidLength);
        }

        let mut bytes = [0u8; Diversifier::LEN];
        bytes.copy_from_slice(&slice[0..16]);
        Ok(Diversifier(bytes))
    }
}

/// Diversifier Key
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct DiversifierKey(pub(crate) [u8; DiversifierKey::LEN]);

impl DiversifierKey {
    pub const LEN: usize = 16;
    pub const LABEL: &'static [u8; 16] = b"Penumbra_DerivDK";

    /// Derive a diversifier key from a spend key.
    /// see reference documentation at:
    /// https://github.com/penumbra-zone/penumbra/blob/main/crates/core/keys/src/keys/fvk.rs#L85
    pub fn derive_from(spend_bytes: &SpendKeyBytes) -> Result<Self, ParserError> {
        let ak = spend_bytes.verification_key()?;
        let nk = spend_bytes.nullifier_key()?;

        let dk_bytes = prf::expand(Self::LABEL, &nk.to_bytes(), ak.as_ref())?;

        let mut bytes = [0u8; Self::LEN];
        bytes.copy_from_slice(&dk_bytes[0..Self::LEN]);

        Ok(Self(bytes))
    }

    pub fn diversifier_for_index(&self, index: &AddressIndex) -> Diversifier {
        let mut key_bytes = [0u8; 16];
        key_bytes.copy_from_slice(&self.0);

        let key = GenericArray::from(key_bytes);

        let mut plaintext_bytes = [0u8; 16];
        plaintext_bytes.copy_from_slice(&index.to_bytes());
        let mut block = GenericArray::from(plaintext_bytes);

        let cipher = Aes128::new(&key);
        cipher.encrypt_block(&mut block);

        let mut ciphertext_bytes = [0u8; 16];
        ciphertext_bytes.copy_from_slice(block.as_slice());
        Diversifier(ciphertext_bytes)
    }

    pub fn index_for_diversifier(&self, diversifier: &Diversifier) -> AddressIndex {
        let mut key_bytes = [0u8; 16];
        key_bytes.copy_from_slice(&self.0);
        let key = GenericArray::from(key_bytes);

        let mut block = GenericArray::from(diversifier.0);

        let cipher = Aes128::new(&key);
        cipher.decrypt_block(&mut block);

        let mut index_bytes = [0; Diversifier::LEN];
        index_bytes.copy_from_slice(&block);

        AddressIndex {
            account: u32::from_le_bytes(
                index_bytes[0..4].try_into().expect("can form 4 byte array"),
            ),
            randomizer: index_bytes[4..16]
                .try_into()
                .expect("can form 12 byte array"),
        }
    }

    /// Convert the key to a le byte array representation
    pub fn to_bytes(self) -> [u8; Self::LEN] {
        self.0
    }
}

impl TryFrom<&SpendKeyBytes> for DiversifierKey {
    type Error = ParserError;

    fn try_from(value: &SpendKeyBytes) -> Result<Self, Self::Error> {
        Self::derive_from(value)
    }
}
