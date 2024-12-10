use crate::constants::FVK_LEN;
use crate::keys::{fvk::FullViewingKey, nk::NullifierKey};
use crate::zxerror::ZxErr;
use decaf377::Fq;
use decaf377_rdsa::{SpendAuth, VerificationKey};
extern "C" {
    pub fn crypto_getFvkBytes(fvk: *mut u8, len: u16) -> ZxErr;
}

pub fn c_fvk_bytes() -> Result<FullViewingKey, ZxErr> {
    unsafe {
        let mut fvk_bytes = [0u8; FVK_LEN];
        let err = crypto_getFvkBytes(fvk_bytes.as_mut_ptr(), fvk_bytes.len() as u16);

        let ak_bytes: [u8; 32] = fvk_bytes[0..32].try_into().unwrap();
        let nk_bytes: [u8; 32] = fvk_bytes[32..64].try_into().unwrap();
        let ak = VerificationKey::<SpendAuth>::try_from(ak_bytes.as_ref()).unwrap();
        let nk = NullifierKey(Fq::from_le_bytes_mod_order(nk_bytes.as_ref()));
        let fvk = FullViewingKey::from_components(ak, nk).unwrap();

        match err {
            ZxErr::Ok => Ok(FullViewingKey::from(fvk)),
            _ => Err(err),
        }
    }
}
