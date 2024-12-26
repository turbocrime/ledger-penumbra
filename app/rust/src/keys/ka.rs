//! The Key agreement implementations taken from Penumbra code base.

use zeroize::Zeroize;

use crate::ParserError;

#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct Public(pub [u8; 32]);

#[derive(Clone, Zeroize, PartialEq, Eq)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
#[zeroize(drop)]
pub struct Secret(decaf377::Fr);

#[derive(Clone, Zeroize, PartialEq, Eq)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
#[zeroize(drop)]
pub struct SharedSecret(pub [u8; 32]);

impl Secret {
    /// Generate a new secret key using the provided `decaf377` field element.
    /// Meant to be used with the SigningKey abstraction.
    pub fn new_from_field(sk: decaf377::Fr) -> Self {
        Self(sk)
    }

    /// Derive a diversified public key for this secret key, using the provided
    /// `diversified_generator`.
    ///
    /// Since key agreement does not depend on the basepoint, only on the secret
    /// key and the public key, a single secret key can correspond to many
    /// different (unlinkable) public keys.
    pub fn diversified_public(&self, diversified_generator: &decaf377::Element) -> Public {
        Public((self.0 * diversified_generator).vartime_compress().into())
    }

    /// Convert this shared secret to bytes.
    ///
    /// Convenience wrapper around an [`Into`] impl.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }

    /// Perform key agreement with the provided public key.
    ///
    /// Fails if the provided public key is invalid.
    pub fn key_agreement_with(&self, other: &Public) -> Result<SharedSecret, ParserError> {
        let pk = decaf377::Encoding(other.0)
            .vartime_decompress()
            .map_err(|_| ParserError::InvalidPubkeyEncoding)?;

        Ok(SharedSecret((self.0 * pk).vartime_compress().into()))
    }
}

impl TryFrom<&[u8]> for Public {
    type Error = ParserError;

    fn try_from(slice: &[u8]) -> Result<Public, Self::Error> {
        let bytes: [u8; 32] = slice.try_into().map_err(|_| ParserError::InvalidLength)?;
        Ok(Public(bytes))
    }
}

impl TryFrom<&[u8]> for Secret {
    type Error = ParserError;

    fn try_from(slice: &[u8]) -> Result<Secret, Self::Error> {
        let bytes: [u8; 32] = slice.try_into().map_err(|_| ParserError::InvalidLength)?;
        bytes.try_into()
    }
}

impl TryFrom<[u8; 32]> for Secret {
    type Error = ParserError;
    fn try_from(bytes: [u8; 32]) -> Result<Secret, ParserError> {
        let x = decaf377::Fr::from_bytes_checked(&bytes).map_err(|_| ParserError::InvalidLength)?;
        Ok(Secret(x))
    }
}
