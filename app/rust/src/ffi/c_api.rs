use crate::keys::fvk::FullViewingKey;
use crate::zxerror::ZxErr;

#[cfg(all(
    not(test),
    not(feature = "clippy"),
    not(feature = "fuzzing"),
    not(feature = "cpp_tests")
))]
extern "C" {
    pub fn crypto_getFvkBytes(fvk: *mut u8, len: u16) -> ZxErr;
}

#[cfg(all(
    not(test),
    not(feature = "clippy"),
    not(feature = "fuzzing"),
    not(feature = "cpp_tests")
))]
pub fn c_fvk_bytes() -> Result<FullViewingKey, ZxErr> {
    use crate::constants::FVK_LEN;
    use crate::keys::nk::NullifierKey;
    use decaf377::Fq;
    use decaf377_rdsa::{SpendAuth, VerificationKey};

    unsafe {
        let mut fvk_bytes = [0u8; FVK_LEN];
        let err = crypto_getFvkBytes(fvk_bytes.as_mut_ptr(), fvk_bytes.len() as u16);

        let ak_bytes: [u8; 32] = fvk_bytes[0..32]
            .try_into()
            .map_err(|_| ZxErr::InvalidCryptoSettings)?;
        let nk_bytes: [u8; 32] = fvk_bytes[32..64]
            .try_into()
            .map_err(|_| ZxErr::InvalidCryptoSettings)?;
        let ak = VerificationKey::<SpendAuth>::try_from(ak_bytes.as_ref())
            .map_err(|_| ZxErr::InvalidCryptoSettings)?;
        let nk = NullifierKey(Fq::from_le_bytes_mod_order(nk_bytes.as_ref()));
        let fvk =
            FullViewingKey::from_components(ak, nk).map_err(|_| ZxErr::InvalidCryptoSettings)?;

        match err {
            ZxErr::Ok => Ok(fvk),
            _ => Err(err),
        }
    }
}

#[cfg(any(test, feature = "clippy", feature = "fuzzing", feature = "cpp_tests"))]
pub fn c_fvk_bytes() -> Result<FullViewingKey, ZxErr> {
    use crate::keys::spend_key::SpendKeyBytes;

    const SK_BYTES_RAW: [u8; 32] = [
        0xa1, 0xff, 0xba, 0x0c, 0x37, 0x93, 0x1f, 0x0a, 0x62, 0x61, 0x37, 0x52, 0x0d, 0xa6, 0x50,
        0x63, 0x2d, 0x35, 0x85, 0x3b, 0xf5, 0x91, 0xb3, 0x6b, 0xb4, 0x28, 0x63, 0x0a, 0x4d, 0x87,
        0xc4, 0xdc,
    ];

    let sk = SpendKeyBytes::from(SK_BYTES_RAW);

    FullViewingKey::derive_from(&sk).map_err(|_| ZxErr::InvalidCryptoSettings)
}
