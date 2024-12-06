use crate::constants::MAX_CLUE_SUBKEYS;
use crate::ParserError;
use decaf377::Fr;
/// Bytes representing a clue key corresponding to some
/// [`DetectionKey`](crate::DetectionKey).
///
/// This type is a refinement type for plain bytes, and is suitable for use in
/// situations where clue key might or might not actually be used.  This saves
/// computation; at the point that a clue key will be used to create a [`Clue`],
/// it can be expanded to an [`ExpandedClueKey`].
#[derive(Copy, Clone, PartialEq, Eq)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct ClueKey(pub [u8; 32]);

#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct ExpandedClueKey {
    root_pub: decaf377::Element,
    root_pub_enc: decaf377::Encoding,
    subkeys: [decaf377::Element; MAX_CLUE_SUBKEYS],
    subkey_index: u8,
}

#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct Clue(pub(crate) [u8; 68]);

impl Default for Clue {
    fn default() -> Self {
        Clue([0u8; 68])
    }
}

impl ClueKey {
    /// Validate and expand this clue key encoding.
    ///
    /// # Errors
    ///
    /// Fails if the bytes don't encode a valid clue key.
    pub fn expand(&self) -> Result<ExpandedClueKey, ParserError> {
        ExpandedClueKey::new(self)
    }

    /// Expand this clue key encoding.
    ///
    /// This method always results in a valid clue key, though the clue key may not have
    /// a known detection key.
    pub fn expand_infallible(&self) -> ExpandedClueKey {
        let mut counter = 0u32;
        loop {
            counter += 1;
            let ck_fq_incremented =
                decaf377::Fq::from_le_bytes_mod_order(&self.0) + decaf377::Fq::from(counter);
            let ck: ClueKey = ClueKey(ck_fq_incremented.to_bytes());

            if let Ok(eck) = ck.expand() {
                return eck;
            }
        }
    }
}

impl TryFrom<&[u8]> for ClueKey {
    type Error = ParserError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() == 32 {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes[0..32]);
            Ok(ClueKey(arr))
        } else {
            Err(Self::Error::InvalidClueKey)
        }
    }
}

impl ExpandedClueKey {
    pub fn new(clue_key: &ClueKey) -> Result<Self, ParserError> {
        let root_pub_enc = decaf377::Encoding(clue_key.0);
        let root_pub = root_pub_enc
            .vartime_decompress()
            .map_err(|_| ParserError::InvalidAddress)?;
        let subkeys = [decaf377::Element::GENERATOR; 10];
        let subkey_index = 0;

        Ok(ExpandedClueKey {
            root_pub,
            root_pub_enc,
            subkeys,
            subkey_index,
        })
    }

    // Checks that the expanded clue key has at least `precision` subkeys
    fn ensure_at_least(&mut self, precision: usize) -> Result<(), ParserError> {
        // The cached expansion is large enough to accommodate the specified precision.
        if precision <= self.subkey_index as usize {
            return Ok(());
        }

        for i in self.subkey_index as usize..precision {
            let hash = blake2b_simd::Params::default()
                .personal(b"decaf377-fmd.hkd")
                .to_state()
                .update(&self.root_pub_enc.0)
                .update(&[i as u8])
                .finalize();
            let x = Fr::from_le_bytes_mod_order(hash.as_bytes());
            let x_element = x * decaf377::Element::GENERATOR;
            let subkey = self.root_pub + x_element;

            self.subkeys[i] = subkey;
        }

        self.subkey_index = precision as u8;

        Ok(())
    }

    pub fn create_clue_deterministic(
        &mut self,
        precision: u8,
        rseed: [u8; 32],
    ) -> Result<Clue, ParserError> {
        if precision > MAX_CLUE_SUBKEYS as u8 {
            return Err(ParserError::InvalidPrecision);
        }
        let precision_bits = precision as usize;
        self.ensure_at_least(precision_bits)?;

        let r = self.compute_hash(b"decaf377-fmd.rdv", &rseed);
        let z = self.compute_hash(b"decaf377-fmd.zdv", &rseed);

        let p = r * decaf377::Element::GENERATOR;
        let p_encoding = p.vartime_compress();
        let q = z * decaf377::Element::GENERATOR;
        let q_encoding = q.vartime_compress();

        let mut ctxts = [0u8; 3];
        let xs = self.subkeys;

        for i in 0..precision_bits {
            let r_xi = (r * xs[i]).vartime_compress();

            let key_i = blake2b_simd::Params::default()
                .personal(b"decaf377-fmd.bit")
                .to_state()
                .update(&p_encoding.0)
                .update(&r_xi.0)
                .update(&q_encoding.0)
                .finalize()
                .as_bytes()[0]
                & 1;

            let ctxt_i = key_i ^ 1u8;
            if ctxt_i != 0 {
                ctxts[i / 8] |= 1 << (i % 8);
            }
        }

        let hash = blake2b_simd::Params::default()
            .personal(b"decaf377-fmd.sca")
            .to_state()
            .update(&p_encoding.0)
            .update(&[precision_bits as u8])
            .update(&ctxts)
            .finalize();

        let m = Fr::from_le_bytes_mod_order(hash.as_bytes());

        let y = (z - m) * r.inverse().expect("random element is nonzero");

        let mut buf = [0u8; 68];
        buf[0..32].copy_from_slice(&p_encoding.0[..]);
        buf[32..64].copy_from_slice(&y.to_bytes()[..]);
        buf[64] = precision_bits as u8;
        buf[65..68].copy_from_slice(&ctxts[..]);

        Ok(Clue(buf))
    }

    fn compute_hash(&self, personal: &[u8], additional_data: &[u8]) -> Fr {
        let hash = blake2b_simd::Params::default()
            .personal(personal)
            .to_state()
            .update(&self.root_pub_enc.0)
            .update(additional_data)
            .finalize();

        Fr::from_le_bytes_mod_order(hash.as_bytes())
    }
}
