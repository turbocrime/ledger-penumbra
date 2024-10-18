use core::{mem::MaybeUninit, ptr::addr_of_mut};

use arrayref::array_ref;
use nom::bytes::complete::take;

use crate::{utils::varint, FromBytes, ParserError};

use super::{Address, Fq, Value};

// proto:
// message Note {
//   asset.v1alpha1.Value value = 1;
//   bytes rseed = 2;
//   keys.v1alpha1.Address address = 3;
// }

#[cfg_attr(test, derive(Debug))]
#[derive(Copy, PartialEq, Eq, Clone)]
pub struct Note<'a> {
    value: Value<'a>,
    rseed: &'a [u8; 32],
    address: Address<'a>,
    transmission_key_s: Fq<'a>,
}

impl<'b> FromBytes<'b> for Note<'b> {
    fn from_bytes_into(
        input: &'b [u8],
        out: &mut MaybeUninit<Self>,
    ) -> Result<&'b [u8], nom::Err<ParserError>> {
        let output = out.as_mut_ptr();

        // Parse the `Value`
        let value = unsafe { &mut *addr_of_mut!((*output).value).cast() };
        let input = Value::from_bytes_into(input, value)?;

        // Parse `rseed` as a 32-byte array
        let (input, _) = varint(input)?;
        let (input, rseed) = take(32usize)(input)?;
        let rseed_ref = array_ref![rseed, 0, 32];

        // Parse `Address`
        let address = unsafe { &mut *addr_of_mut!((*output).address).cast() };
        let input = Address::from_bytes_into(input, address)?;

        // Parse `transmission_key_s` (Fq type)
        let transmission_key_s = unsafe { &mut *addr_of_mut!((*output).transmission_key_s).cast() };
        let input = Fq::from_bytes_into(input, transmission_key_s)?;

        unsafe {
            addr_of_mut!((*output).rseed).write(rseed_ref);
        }

        Ok(input)
    }
}
