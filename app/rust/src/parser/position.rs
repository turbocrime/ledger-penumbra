use core::{mem::MaybeUninit, ptr::addr_of_mut};

use crate::{utils::varint, FromBytes, ParserError};

#[cfg_attr(test, derive(Debug))]
#[derive(Copy, PartialEq, Eq, Clone)]
pub struct Tree {
    pub epoch: u16,
    pub block: u16,
    pub commitment: u16,
}

impl From<u64> for Tree {
    fn from(position: u64) -> Self {
        let epoch = (position >> 32) as u16;
        let block = (position >> 16) as u16;
        let commitment = position as u16;
        Self {
            epoch,
            block,
            commitment,
        }
    }
}

// proto:
// uint64 position = 2;
#[cfg_attr(test, derive(Debug))]
#[derive(Copy, PartialEq, Eq, Clone)]
pub struct Position(pub Tree);

impl<'b> FromBytes<'b> for Position {
    fn from_bytes_into(
        input: &'b [u8],
        out: &mut MaybeUninit<Self>,
    ) -> Result<&'b [u8], nom::Err<ParserError>> {
        let out = out.as_mut_ptr();
        let (input, position) = varint(input)?;

        let tree = Tree::from(position);

        unsafe {
            addr_of_mut!((*out).0).write(tree);
        }

        Ok(input)
    }
}
