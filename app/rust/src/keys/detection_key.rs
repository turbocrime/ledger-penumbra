use decaf377::{Element, Fr};

use crate::{utils::prf, ParserError};

use super::{spend_key::SpendKeyBytes, ClueKey};

pub struct DetectionKey {
    /// The detection key.
    pub(crate) dtk: Fr,
}

impl DetectionKey {
    pub const LABEL: &'static [u8; 16] = b"PenumbraExpndFMD";
    pub const MAX_PRECISION: usize = 24;

    pub fn derive_from(spend_key: &SpendKeyBytes) -> Result<Self, ParserError> {
        // dtk_d = from_le_bytes(prf_expand(b"PenumbraExpndFMD", to_le_bytes(ivk), d))
        let ivk = spend_key.ivk()?;
        let dk = spend_key.diversifier_key()?;

        let fr = prf::expand_fr::expand_ff(Self::LABEL, &ivk.ivk.to_bytes(), &dk.to_bytes())?;

        Ok(Self::from_field(fr))
    }

    pub fn from_field(dtk: Fr) -> Self {
        Self { dtk }
    }

    /// Serialize this detection key to bytes.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.dtk.to_bytes()
    }

    /// Deserialize a detection key from bytes.
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, ParserError> {
        let dtk = Fr::from_bytes_checked(bytes).map_err(|_| ParserError::InvalidDetectionKey)?;
        Ok(Self::from_field(dtk))
    }

    /// Obtain the clue key corresponding to this detection key.
    pub fn clue_key(&self) -> ClueKey {
        // there is not a decaf377::basepoint() function,
        // so use the inner implementation instead
        let base = Element::GENERATOR;
        let mul = (self.dtk * base).vartime_compress();
        ClueKey(mul.0)
    }

    // Use this detection key to examine the given `clue`, returning `true` if the
    // clue was possibly sent to this detection key's clue key.
    //
    // This test has false positives, but no false negatives.
    //
    // This function executes in constant time with respect to the detection
    // key material, but short-circuits to return early on a false detection.
    // #[allow(non_snake_case)]
    // pub fn examine(&self, clue: &Clue) -> bool {
    //     let P_encoding = decaf377::Encoding::try_from(&clue.0[0..32]).expect("slice is right len");
    //
    //     let P = if let Ok(P) = P_encoding.vartime_decompress() {
    //         P
    //     } else {
    //         // Invalid P encoding => not a match
    //         return false;
    //     };
    //
    //     let y = if let Ok(y) = Fr::deserialize_compressed(&clue.0[32..64]) {
    //         y
    //     } else {
    //         // Invalid y encoding => not a match
    //         return false;
    //     };
    //
    //     // Reject P = 0 or y = 0, as these never occur in well-formed clues; as
    //     // noted in the OpenPrivacy implementation, these could allow clues to
    //     // match any detection key.
    //     // https://docs.rs/fuzzytags/0.6.0/src/fuzzytags/lib.rs.html#348-351
    //     if P.is_identity() || y.is_zero() {
    //         return false;
    //     }
    //
    //     let precision_bits = clue.0[64];
    //     let ciphertexts = BitSlice::<u8, order::Lsb0>::from_slice(&clue.0[65..68]);
    //
    //     let m = hash::to_scalar(&P_encoding.0, precision_bits, &clue.0[65..68]);
    //     let Q_bytes = ((y * P) + (m * decaf377::basepoint())).vartime_compress();
    //
    //     for i in 0..(precision_bits as usize) {
    //         let Px_i = (P * self.xs[i]).vartime_compress();
    //         let key_i = hash::to_bit(&P_encoding.0, &Px_i.0, &Q_bytes.0);
    //         let msg_i = (ciphertexts[i] as u8) ^ key_i;
    //         // Short-circuit if we get a zero; this branch is dependent on the
    //         // ephemeral key bit `key_i`, not the long-term key `xs[i]`, so we
    //         // don't risk leaking any long-term secrets through timing channels.
    //         //
    //         // On the other hand, this gives a massive speedup, since we have a
    //         // 1/2 chance of rejecting after 1 iteration, 1/4 chance of
    //         // rejecting after 2 iterations, ..., so (in expectation) we do <= 2
    //         // iterations instead of n iterations.
    //         if msg_i == 0 {
    //             return false;
    //         }
    //     }
    //
    //     // Otherwise, all message bits were 1 and we return true.
    //     true
    // }
}
