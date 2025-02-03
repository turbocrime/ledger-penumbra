use crate::address::{Address, AddressIndex};
use crate::constants::KEY_LEN;
use crate::ffi::c_api::c_fvk_bytes;
use crate::keys::spend_key::SpendKeyBytes;
use crate::parser::bytes::BytesC;
use crate::ParserError;

#[repr(C)]
pub struct Keys {
    skb: [u8; SpendKeyBytes::LEN],
    // fvk is the concatenation of:
    // - verificationKeyBytes (32 bytes)
    // - nullifierKeyBytes (32 bytes)
    fvk: [u8; KEY_LEN * 2],
    // Holds 80-byte raw data
    address: [u8; Address::LEN],
}

#[no_mangle]
/// Use to compute an address and write it back into output
/// argument.
pub unsafe extern "C" fn rs_compute_keys(keys: &mut Keys) -> u32 {
    crate::zlog("rs_compute_keys\x00");

    if let Err(code) = compute_keys(keys) {
        return code as _;
    }

    ParserError::Ok as u32
}

#[no_mangle]
/// Use to compute an address and write it back into output
/// argument.
pub unsafe extern "C" fn rs_compute_address(
    keys: &mut Keys,
    account: u32,
    randomizer: *const u8,
) -> u32 {
    crate::zlog("rs_compute_address\x00");
    let mut addr_idx = AddressIndex::new(account);

    if !randomizer.is_null() {
        let randomizer = core::slice::from_raw_parts(randomizer, AddressIndex::RAND_LEN);
        addr_idx.randomizer.copy_from_slice(randomizer);
    }

    if let Err(code) = compute_address(keys, addr_idx) {
        return code as u32;
    }

    ParserError::Ok as u32
}

#[no_mangle]
/// Use to compute an address and write it back into output
/// argument.
pub unsafe extern "C" fn rs_is_address_visible(
    address: &BytesC,
    is_visible: *mut bool,
    index: *mut u32,
) -> u32 {
    crate::zlog("rs_is_address_visible\x00");

    if is_visible.is_null() || index.is_null() {
        return ParserError::NoData as u32;
    }

    let Ok(fvk) = c_fvk_bytes() else {
        return ParserError::UnexpectedError as u32;
    };

    let Ok(address_bytes) = address.get_bytes() else {
        return ParserError::InvalidAddress as u32;
    };

    let Ok(address) = Address::try_from(address_bytes) else {
        return ParserError::InvalidAddress as u32;
    };

    match fvk.address_index(&address) {
        Some(idx) => {
            *is_visible = true;
            *index = idx.account;
        }
        None => {
            *is_visible = false;
            *index = 0;
        }
    }

    ParserError::Ok as u32
}

fn compute_address(keys: &mut Keys, addr_idx: AddressIndex) -> Result<(), ParserError> {
    let spk = SpendKeyBytes::from(keys.skb);
    let fvk = spk.fvk()?;
    let ivk = fvk.ivk();

    let address = ivk.payment_address(addr_idx).map(|(addr, _)| addr)?;

    // return the f4jumble encoded raw address
    let raw = address.to_bytes()?;

    keys.address.copy_from_slice(&raw);

    Ok(())
}

fn compute_keys(keys: &mut Keys) -> Result<(), ParserError> {
    let spend_key = SpendKeyBytes::from(keys.skb);
    let fvk = spend_key.fvk()?;
    fvk.to_bytes_into(&mut keys.fvk)?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::keys::fvk::FullViewingKey;
    use crate::keys::nk::NullifierKey;
    use crate::keys::spend_key::SpendKeyBytes;
    use decaf377::Fq;
    use decaf377_rdsa::{SpendAuth, VerificationKey};
    const SPEND_KEY: &str = "ff726c71bcec76abc6a88cba71df655b28de6580edbd33c7415fdfded2e422e7";
    const SPEND_ZEMU_KEY: &str = "a1ffba0c37931f0a626137520da650632d35853bf591b36bb428630a4d87c4dc";
    const ACCOUNT_IDX: u32 = 1;
    const EXPECTED_ADDR: &str = "70c4d192ddf3c4cdf97fddc4c4aa07d112b5a7bf6d0810da37ae777990913737babcaa57fd4031d19260d88f1ec0c357a375c289f9943e7efa242ae963abcce749543a22039d687d8a027cb05b33438c";
    const EXPECTED_DIV: &str = "fe8f546c0172716f9efd52eba9074148";
    const EXPECTED_DIVERSIFIER_KEY: &str = "9d2107be5bfa0c07a7f870e216f185d9";
    const EXPECTED_DIV_FOR_INDEX: &str = "fe8f546c0172716f9efd52eba9074148";
    // detection key
    const EXPECTED_DTK_D: &str = "47eed67e862907275f4062cbdd80c97a5720b04696ef49a311444c1c8bce0304";
    const EXPECTED_CLUE_KEY: &str =
        "a0a9b1b8a39a0fe0eaacc74d1e84399f74c94f805d6ee83f38609f63aa85bf01 ";

    const EXPECTED_PUBLIC: &str =
        "d8e051b4671997771d22e5b9203fc337055e0736660c922372692b7b8dd7ac07";

    const EXPECTED_FVK: &str = "b8380bd5aa798359cb70a1496e8b41d1b557e0669da158215c00ccf6d3fd6f12b89201d8f297f9898b357e0175699218b2121cbf0f444fe63a476805bbe8fb0d";

    #[test]
    fn verify_addr() {
        let addr_idx = AddressIndex::new(ACCOUNT_IDX);
        let key_bytes = hex::decode(SPEND_KEY).unwrap();
        let expected_addr = hex::decode(EXPECTED_ADDR).unwrap();

        let mut keys = Keys {
            skb: [0; SpendKeyBytes::LEN],
            fvk: [0; KEY_LEN * 2],
            address: [0; Address::LEN],
        };

        keys.skb.copy_from_slice(&key_bytes);

        compute_address(&mut keys, addr_idx).unwrap();

        assert_eq!(keys.address, expected_addr.as_slice());
    }

    #[test]
    fn verify_fvk() {
        let key_bytes = hex::decode(SPEND_KEY).unwrap();

        let mut keys = Keys {
            skb: [0; SpendKeyBytes::LEN],
            fvk: [0; KEY_LEN * 2],
            address: [0; Address::LEN],
        };

        keys.skb.copy_from_slice(&key_bytes);

        compute_keys(&mut keys).unwrap();

        let s = hex::encode(keys.fvk);

        assert_eq!(s, EXPECTED_FVK);
    }

    #[test]
    fn get_fvk_from_bytes() {
        let key_bytes = hex::decode(SPEND_ZEMU_KEY).unwrap();
        let mut key_bytes_array = [0u8; 32];
        key_bytes_array.copy_from_slice(&key_bytes);
        let spend_key = SpendKeyBytes::from(key_bytes_array);

        let fvk = spend_key.fvk().unwrap();

        let mut keys = Keys {
            skb: [0; SpendKeyBytes::LEN],
            fvk: [0; KEY_LEN * 2],
            address: [0; Address::LEN],
        };

        fvk.to_bytes_into(&mut keys.fvk).unwrap();

        let s = hex::encode(keys.fvk);

        let ak_bytes: [u8; 32] = keys.fvk[0..32].try_into().unwrap();
        let nk_bytes: [u8; 32] = keys.fvk[32..64].try_into().unwrap();
        let ak = VerificationKey::<SpendAuth>::try_from(ak_bytes.as_ref()).unwrap();
        let nk = NullifierKey(Fq::from_le_bytes_mod_order(nk_bytes.as_ref()));
        let fvk_2 = FullViewingKey::from_components(ak, nk).unwrap();

        let mut keys_2 = Keys {
            skb: [0; SpendKeyBytes::LEN],
            fvk: [0; KEY_LEN * 2],
            address: [0; Address::LEN],
        };

        fvk_2.to_bytes_into(&mut keys_2.fvk).unwrap();

        let s_2 = hex::encode(keys_2.fvk);

        assert_eq!(s, s_2);
    }
}
