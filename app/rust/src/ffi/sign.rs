use decaf377::Fr;
use decaf377_rdsa::{Signature, SigningKey, SpendAuth};

use crate::{
    constants::SIGNATURE_LEN, keys::spend_key::SpendKeyBytes, parser::BytesC, ParserError,
};

/// # Safety
/// This function is unsafe because depends on passed raw pointers from C
#[no_mangle]
pub unsafe extern "C" fn rs_sign_spend(
    effect_hash: &BytesC,
    randomizer: &BytesC,
    spend_key: &SpendKeyBytes,
    signature: *mut u8,
    len: u16,
) -> u32 {
    if len < SIGNATURE_LEN as u16 {
        return ParserError::InvalidLength as u32;
    }

    match sign_spend(effect_hash, randomizer, spend_key) {
        Ok(sk) => {
            let signature = core::slice::from_raw_parts_mut(signature, len as usize);
            signature.copy_from_slice(sk.to_bytes().as_ref());

            ParserError::Ok as u32
        }
        Err(e) => e as u32,
    }
}

pub fn randomized_signing_key(
    spend_key: &SpendKeyBytes,
    randomizer: &BytesC,
) -> Result<SigningKey<SpendAuth>, ParserError> {
    let sk = spend_key.signing_key()?;

    let randomizer: &[u8] = randomizer.into();

    let randomizer = Fr::from_le_bytes_mod_order(randomizer);

    Ok(sk.randomize(&randomizer))
}

pub fn sign_spend(
    effect_hash: &BytesC,
    randomizer: &BytesC,
    spend_key: &SpendKeyBytes,
) -> Result<Signature<SpendAuth>, ParserError> {
    use rand_chacha::{rand_core::SeedableRng, ChaCha20Rng};

    let sk = randomized_signing_key(spend_key, randomizer)?;

    let effect_hash = effect_hash.into();
    // TODO: check if seed must be taken as an argument, for now lets use the passed randomizer
    let seed = randomizer.into_array()?;
    let mut rng = ChaCha20Rng::from_seed(seed);

    Ok(sk.sign(&mut rng, effect_hash))
}
