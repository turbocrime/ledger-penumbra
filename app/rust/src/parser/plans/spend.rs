use core::{mem::MaybeUninit, ptr::addr_of_mut};

use crate::{
    parser::{Fq, Fr, Note, Position},
    FromBytes, ParserError,
};
// proto:
// message SpendPlan {
//   // The plaintext note we plan to spend.
//   Note note = 1;
//   // The position of the note we plan to spend.
//   uint64 position = 2;
//   // The randomizer to use for the spend.
//   bytes randomizer = 3;
//   // The blinding factor to use for the value commitment.
//   bytes value_blinding = 4;
//   // The first blinding factor to use for the ZK spend proof.
//   bytes proof_blinding_r = 5;
//   // The second blinding factor to use for the ZK spend proof.
//   bytes proof_blinding_s = 6;
// }
#[cfg_attr(test, derive(Debug))]
#[derive(Copy, PartialEq, Eq, Clone)]
pub struct SpendPlan<'a> {
    pub note: Note<'a>,
    pub position: Position,
    pub randomizer: Fr<'a>,
    pub value_blinding: Fr<'a>,
    pub proof_blinding_r: Fq<'a>,
    pub proof_blinding_s: Fq<'a>,
}

impl<'b> FromBytes<'b> for SpendPlan<'b> {
    fn from_bytes_into(
        input: &'b [u8],
        out: &mut MaybeUninit<Self>,
    ) -> Result<&'b [u8], nom::Err<ParserError>> {
        let output = out.as_mut_ptr();

        // Parsing each field sequentially
        // Parse `note`
        let note = unsafe { &mut *addr_of_mut!((*output).note).cast() };
        let input = Note::from_bytes_into(input, note)?;

        // Parse `position`
        let position = unsafe { &mut *addr_of_mut!((*output).position).cast() };
        let input = Position::from_bytes_into(input, position)?;

        // Parse `randomizer`
        let randomizer = unsafe { &mut *addr_of_mut!((*output).randomizer).cast() };
        let input = Fr::from_bytes_into(input, randomizer)?;

        // Parse `value_blinding`
        let value = unsafe { &mut *addr_of_mut!((*output).value_blinding).cast() };
        let input = Fr::from_bytes_into(input, value)?;

        // Parse `proof_blinding_r`
        let proof = unsafe { &mut *addr_of_mut!((*output).proof_blinding_r).cast() };
        let input = Fq::from_bytes_into(input, proof)?;

        // Parse `proof_blinding_s`
        let proof = unsafe { &mut *addr_of_mut!((*output).proof_blinding_s).cast() };
        let input = Fq::from_bytes_into(input, proof)?;

        Ok(input)
    }
}
