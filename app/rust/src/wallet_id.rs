/// The hash of a full viewing key, used as an account identifier.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WalletId(pub [u8; 32]);
