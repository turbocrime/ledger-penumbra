use crate::{utils::varint, FromBytes, ParserError};

use core::ptr::addr_of_mut;
use std::mem::MaybeUninit;

// The quantity of a particular Asset. Represented as a 128-bit unsigned integer,
// split over two fields, `lo` and `hi`, representing the low- and high-order bytes
// of the 128-bit value, respectively. Clients must assemble these bits in their
// implementation into a `uint128` or comparable data structure, in order to model
// the Amount accurately.
// message Amount {
//   uint64 lo = 1;
//   uint64 hi = 2;
// }

#[cfg_attr(test, derive(Debug))]
#[derive(Copy, PartialEq, Eq, Clone)]
pub struct Amount {
    pub lo: u64,
    pub hi: u64,
}

impl Amount {
    pub fn new(lo: u64, hi: u64) -> Self {
        Self { lo, hi }
    }
}

impl<'b> FromBytes<'b> for Amount {
    fn from_bytes_into(
        input: &'b [u8],
        out: &mut MaybeUninit<Self>,
    ) -> Result<&'b [u8], nom::Err<ParserError>> {
        let out = out.as_mut_ptr();
        let (rem, lo) = varint(input)?;
        let (rem, hi) = varint(rem)?;

        unsafe {
            addr_of_mut!((*out).lo).write(lo);
            addr_of_mut!((*out).hi).write(hi);
        }

        Ok(rem)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::std::vec::Vec;
    use crate::utils::encode_varint;

    #[test]
    fn test_amount_from_bytes() {
        // Example encoded data for Amount, adjust as per your actual encoded data
        let lo = 123456789; // Example low-order value
        let hi = 987654321; // Example high-order value

        let encoded_lo = encode_varint(lo);
        let encoded_hi = encode_varint(hi);
        let mut encoded_data = Vec::new();
        encoded_data.extend(encoded_lo);
        encoded_data.extend(encoded_hi);

        let (rem, amount) = Amount::from_bytes(&encoded_data).unwrap();

        assert!(rem.is_empty());

        assert_eq!(amount.lo, lo);
        assert_eq!(amount.hi, hi);
    }
}
