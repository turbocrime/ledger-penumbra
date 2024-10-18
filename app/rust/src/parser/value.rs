use core::{mem::MaybeUninit, ptr::addr_of_mut};

use crate::{FromBytes, ParserError};

use super::{Amount, AssetId};

// proto:
// message Value {
//   core.num.v1alpha1.Amount amount = 1;
//   AssetId asset_id = 2;
// }

#[cfg_attr(test, derive(Debug))]
#[derive(Copy, PartialEq, Eq, Clone)]
pub struct Value<'a> {
    amount: Amount,
    asset_id: AssetId<'a>,
}

impl<'b> FromBytes<'b> for Value<'b> {
    fn from_bytes_into(
        input: &'b [u8],
        out: &mut MaybeUninit<Self>,
    ) -> Result<&'b [u8], nom::Err<ParserError>> {
        let output = out.as_mut_ptr();
        // First, parse the `Amount`
        let amount = unsafe { &mut *addr_of_mut!((*output).amount).cast() };
        let input = Amount::from_bytes_into(input, amount)?;

        // Then, parse the `AssetId`
        let asset_id = unsafe { &mut *addr_of_mut!((*output).asset_id).cast() };
        let input = AssetId::<'b>::from_bytes_into(input, asset_id)?;

        Ok(input)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use std::vec::Vec;

    #[test]
    fn test_value_from_bytes() {
        let asset_encoded = [
            10, 32, 98, 101, 10, 229, 197, 119, 125, 22, 96, 204, 23, 252, 212, 244, 143, 106, 102,
            185, 164, 194, 175, 205, 72, 246, 236, 175, 245, 90, 222, 240, 190, 239,
        ];

        let amount_encoded = [149, 154, 239, 58, 177, 209, 249, 214, 3];
        let hex_str = "62650ae5c5777d1660cc17fcd4f48f6a66b9a4c2afcd48f6ecaff55adef0beef";
        let asset_id_bytes = hex::decode(hex_str).expect("Invalid hex string");

        let mut encoded_data = Vec::new();
        encoded_data.extend(amount_encoded);
        encoded_data.extend(asset_encoded);

        let (rem, value) = Value::from_bytes(&encoded_data).unwrap();

        assert!(rem.is_empty());

        // Check values
        let lo = 123456789;
        let hi = 987654321;

        assert_eq!(value.amount.lo, lo);
        assert_eq!(value.amount.hi, hi);

        assert_eq!(&value.asset_id.0[..], asset_id_bytes.as_slice());
    }
}
