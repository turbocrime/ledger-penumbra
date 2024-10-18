// TODO: decaf377 define Fq and Fr as [u64; N]
// we adjust this to u32 assuming penumbra team will come up with this alternative
#[cfg_attr(test, derive(Debug))]
#[derive(Copy, PartialEq, Eq, Clone)]
pub struct BigInt<const N: usize>(pub [u32; N]);
