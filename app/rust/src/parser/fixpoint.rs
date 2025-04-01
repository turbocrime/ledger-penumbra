use crate::parser::amount::Amount;
use crate::parser::ParserError;
use ethnum::U256;
use std::ops::Mul;

#[derive(Copy, Clone)]
pub struct U128x128(U256);

impl U128x128 {
    /// Encode this number as a 32-byte array.
    ///
    /// The encoding has the property that it preserves ordering, i.e., if `x <=
    /// y` (with numeric ordering) then `x.to_bytes() <= y.to_bytes()` (with the
    /// lex ordering on byte strings).
    pub fn to_bytes(self) -> [u8; 32] {
        // The U256 type has really weird endianness handling -- e.g., it reverses
        // the endianness of the inner u128s (??) -- so just do it manually.
        let mut bytes = [0u8; 32];
        let (hi, lo) = self.0.into_words();
        bytes[0..16].copy_from_slice(&hi.to_be_bytes());
        bytes[16..32].copy_from_slice(&lo.to_be_bytes());
        bytes
    }

    /// Decode this number from a 32-byte array.
    pub fn from_bytes(bytes: [u8; 32]) -> Result<Self, ParserError> {
        // See above.
        let hi = u128::from_be_bytes(
            bytes[0..16]
                .try_into()
                .map_err(|_| ParserError::InvalidLength)?,
        );
        let lo = u128::from_be_bytes(
            bytes[16..32]
                .try_into()
                .map_err(|_| ParserError::InvalidLength)?,
        );
        Ok(Self(U256::from_words(hi, lo)))
    }

    /// Multiply an amount by this fraction, then round down.
    pub fn apply_to_amount(self, rhs: &Amount) -> Result<Amount, ParserError> {
        let mul = (Self::from(rhs) * self)?;
        let out = mul.round_down().try_into()?;
        Ok(out)
    }

    /// Checks whether this number is integral, i.e., whether it has no fractional part.
    pub fn is_integral(&self) -> bool {
        let fractional_word = self.0.into_words().1;
        fractional_word == 0
    }

    /// Rounds the number down to the nearest integer.
    pub fn round_down(self) -> Self {
        let integral_word = self.0.into_words().0;
        Self(U256::from_words(integral_word, 0u128))
    }

    /// Performs checked multiplication, returning `Ok` if no overflow occurred.
    pub fn checked_mul(self, rhs: &Self) -> Result<Self, ParserError> {
        // It's important to use `into_words` because the `U256` type has an
        // unsafe API that makes the limb ordering dependent on the host
        // endianness.
        let (x1, x0) = self.0.into_words();
        let (y1, y0) = rhs.0.into_words();
        let x0 = U256::from(x0);
        let x1 = U256::from(x1);
        let y0 = U256::from(y0);
        let y1 = U256::from(y1);

        // x = (x0*2^-128 + x1)*2^128
        // y = (y0*2^-128 + y1)*2^128
        // x*y        = (x0*y0*2^-256 + (x0*y1 + x1*y0)*2^-128 + x1*y1)*2^256
        // x*y*2^-128 = (x0*y0*2^-256 + (x0*y1 + x1*y0)*2^-128 + x1*y1)*2^128
        //               ^^^^^
        //               we drop the low 128 bits of this term as rounding error

        let x0y0 = x0 * y0; // cannot overflow, widening mul
        let x0y1 = x0 * y1; // cannot overflow, widening mul
        let x1y0 = x1 * y0; // cannot overflow, widening mul
        let x1y1 = x1 * y1; // cannot overflow, widening mul

        let (x1y1_hi, _x1y1_lo) = x1y1.into_words();
        if x1y1_hi != 0 {
            return Err(ParserError::Overflow);
        }

        x1y1.checked_shl(128)
            .and_then(|acc| acc.checked_add(x0y1))
            .and_then(|acc| acc.checked_add(x1y0))
            .and_then(|acc| acc.checked_add(x0y0 >> 128))
            .map(U128x128)
            .ok_or(ParserError::Overflow)
    }
}

impl TryFrom<[u8; 32]> for U128x128 {
    type Error = ParserError;
    fn try_from(value: [u8; 32]) -> Result<Self, Self::Error> {
        Self::from_bytes(value)
    }
}

impl TryFrom<&[u8]> for U128x128 {
    type Error = ParserError;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        <[u8; 32]>::try_from(value)
            .map_err(|_| ParserError::InvalidLength)?
            .try_into()
    }
}

impl TryFrom<U128x128> for u128 {
    type Error = ParserError;
    fn try_from(value: U128x128) -> Result<Self, Self::Error> {
        match value.is_integral() {
            true => Ok(value.0.into_words().0),
            false => Err(ParserError::NonIntegral),
        }
    }
}

impl From<u128> for U128x128 {
    fn from(value: u128) -> Self {
        Self(U256::from_words(value, 0))
    }
}

impl Mul<U128x128> for U128x128 {
    type Output = Result<U128x128, ParserError>;
    fn mul(self, rhs: U128x128) -> Self::Output {
        self.checked_mul(&rhs)
    }
}
