use crate::parser::bytes::BytesC;
use decaf377_rdsa::{SpendAuth, VerificationKeyBytes};


pub struct IdentityKey(pub VerificationKeyBytes<SpendAuth>);

#[repr(C)]
#[derive(Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct IdentityKeyC {
    pub ik: BytesC,
}


