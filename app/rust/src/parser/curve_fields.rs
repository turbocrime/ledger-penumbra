use core::{mem::MaybeUninit, ptr::addr_of_mut};

use arrayref::array_ref;
use nom::bytes::complete::take;

use crate::{utils::varint, FromBytes, ParserError};

// TODO: This types would be wrappers around Fq and Fr types provided by the penumbra team
// who aims to come up with a no_std/no_alloc compatible decaf377-rdsa library.
// similar definition:
// pub type Fr = Fp256<MontBackend<FrConfig, 4>>; where Fp is BigInt<u64; N>, N=4
// so total is 256 bits -> 32-bytes
#[cfg_attr(test, derive(Debug))]
#[derive(Copy, PartialEq, Eq, Clone)]
pub struct Fr<'a>(pub &'a [u8; 32]);

// TODO: confirm they are the same
pub type Fq<'a> = Fr<'a>;

impl<'b> FromBytes<'b> for Fr<'b> {
    fn from_bytes_into(
        input: &'b [u8],
        out: &mut MaybeUninit<Self>,
    ) -> Result<&'b [u8], nom::Err<ParserError>> {
        let out = out.as_mut_ptr();

        let (input, _) = varint(input)?; // Parse field number and wire type
        let (input, len) = varint(input)?; // Parse length

        if len as usize != 32 {
            return Err(ParserError::InvalidLength.into());
        }

        let (input, bytes) = take(len as usize)(input)?;
        let bytes_ref = array_ref![bytes, 0, 32];

        unsafe {
            addr_of_mut!((*out).0).write(bytes_ref);
        }

        Ok(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_bytes_valid_input() {
        let input = [
            0x0A, 0x20, // Prepended bytes: field number + wire type, and length
            0x01, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
            0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
            0x20, 0x20, 0x20, 0x20,
        ];
        let mut out = MaybeUninit::<Fr>::uninit();

        Fr::from_bytes_into(&input, &mut out).unwrap();
    }

    #[test]
    fn test_from_bytes_invalid_length() {
        let input = [
            0x01, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
            0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
            0x20, 0x20,
        ];
        let mut out = MaybeUninit::<Fr>::uninit();

        let result = Fr::from_bytes_into(&input, &mut out);

        assert!(result.is_err());
    }
}
