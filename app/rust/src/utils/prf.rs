use crate::ParserError;

pub fn expand(label: &'static [u8; 16], key: &[u8], input: &[u8]) -> Result<[u8; 64], ParserError> {
    if key.len() > blake2b_simd::KEYBYTES {
        return Err(ParserError::InvalidKeyLen);
    }
    let mut params = blake2b_simd::Params::new();
    let mut params = params.personal(label);

    // Add keys only if not empty
    // used for computation of detection key
    if !key.is_empty() {
        params = params.key(key);
    }

    let mut output = [0u8; 64];
    let hash = params.hash(input);

    output.copy_from_slice(hash.as_bytes());

    Ok(output)
}

pub mod expand_fp {
    use super::*;
    use decaf377::Fp;

    // pub fn expand_ff<F: PrimeField>(label: &'static [u8; 16], key: &[u8], input: &[u8]) -> F {
    //     F::from_le_bytes_mod_order(expand(label, key, input).as_bytes())
    // }
    pub fn expand_ff(
        label: &'static [u8; 16],
        key: &[u8],
        input: &[u8],
    ) -> Result<Fp, ParserError> {
        Ok(Fp::from_le_bytes_mod_order(
            expand(label, key, input)?.as_ref(),
        ))
    }
}

pub mod expand_fq {
    use decaf377::Fq;

    use super::*;

    // pub fn expand_ff<F: PrimeField>(label: &'static [u8; 16], key: &[u8], input: &[u8]) -> F {
    //     F::from_le_bytes_mod_order(expand(label, key, input).as_bytes())
    // }
    pub fn expand_ff(
        label: &'static [u8; 16],
        key: &[u8],
        input: &[u8],
    ) -> Result<Fq, ParserError> {
        Ok(Fq::from_le_bytes_mod_order(
            expand(label, key, input)?.as_ref(),
        ))
    }
}

pub mod expand_fr {
    use decaf377::Fr;

    use super::*;

    // pub fn expand_ff<F: PrimeField>(label: &'static [u8; 16], key: &[u8], input: &[u8]) -> F {
    //     F::from_le_bytes_mod_order(expand(label, key, input).as_bytes())
    // }
    pub fn expand_ff(
        label: &'static [u8; 16],
        key: &[u8],
        input: &[u8],
    ) -> Result<Fr, ParserError> {
        Ok(Fr::from_le_bytes_mod_order(
            expand(label, key, input)?.as_ref(),
        ))
    }
}
