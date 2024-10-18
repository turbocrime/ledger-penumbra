use decaf377::Fq;

use crate::ParserError;

use super::Public;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct TransmissionKey(pub(crate) Fq);

impl TryFrom<Public> for TransmissionKey {
    type Error = ParserError;
    fn try_from(public: Public) -> Result<Self, Self::Error> {
        Fq::from_bytes_checked(&public.0)
            .map(TransmissionKey)
            .map_err(|_| ParserError::InvalidTxKey)
    }
}
