use arrayref::array_ref;
use core::ptr::addr_of_mut;
use nom::bytes::complete::take;
use std::mem::MaybeUninit;

use crate::{utils::varint, FromBytes, ParserError};

// proto:
// message AssetId {
//   // The bytes of the asset ID.
//   bytes inner = 1;
//
//   // Alternatively, a Bech32m-encoded string representation of the `inner`
//   // bytes.
//   //
//   // NOTE: implementations are not required to support parsing this field.
//   // Implementations should prefer to encode the `inner` bytes in all messages they
//   // produce. Implementations must not accept messages with both `inner` and
//   // `alt_bech32m` set.  This field exists for convenience of RPC users.
//   string alt_bech32m = 2;
//
//   // Alternatively, a base denomination string which should be hashed to obtain the asset ID.
//   //
//   // NOTE: implementations are not required to support parsing this field.
//   // Implementations should prefer to encode the bytes in all messages they
//   // produce. Implementations must not accept messages with both `inner` and
//   // `alt_base_denom` set.  This field exists for convenience of RPC users.
//   string alt_base_denom = 3;
// }

#[cfg_attr(test, derive(Debug))]
#[derive(Copy, PartialEq, Eq, Clone)]
pub struct AssetId<'a>(pub &'a [u8; AssetId::ID_LEN]);

impl<'a> AssetId<'a> {
    const ID_LEN: usize = 32;
}

impl<'b> FromBytes<'b> for AssetId<'b> {
    fn from_bytes_into(
        input: &'b [u8],
        out: &mut MaybeUninit<Self>,
    ) -> Result<&'b [u8], nom::Err<ParserError>> {
        let out = out.as_mut_ptr();

        let (input, _) = varint(input)?; // Parse field number and wire type for `inner`
        let (input, len) = varint(input)?; // Parse length of `inner`

        if len as usize != AssetId::ID_LEN {
            return Err(ParserError::ValueOutOfRange.into());
        }

        let (input, inner_bytes) = take(AssetId::ID_LEN)(input)?;
        let id_bytes = array_ref![inner_bytes, 0, 32];

        unsafe {
            addr_of_mut!((*out).0).write(id_bytes);
        }

        Ok(input)
    }
}

#[cfg(test)]
mod tests {
    use std::println;

    use crate::utils::encode_bytes;

    use super::*;

    #[test]
    fn test_asset_id_from_bytes() {
        // Convert the hex string to bytes
        let hex_str = "62650ae5c5777d1660cc17fcd4f48f6a66b9a4c2afcd48f6ecaff55adef0beef";
        let asset_id_bytes = hex::decode(hex_str).expect("Invalid hex string");

        let encoded_data = encode_bytes(&asset_id_bytes);
        println!("asset_encoded: {:?}", encoded_data);

        let (rem, id) = AssetId::from_bytes(&encoded_data).unwrap();

        assert!(rem.is_empty());

        assert_eq!(&id.0[..], asset_id_bytes.as_slice());
    }
}
