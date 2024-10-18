use crate::wallet_id::WalletId;

use super::{Address, AddressIndex};

/// A view of a Penumbra address, either an opaque payment address or an address
/// with known structure.
///
/// This type allows working with addresses and address indexes without knowing
/// the corresponding FVK.
#[derive(Clone, Copy, PartialEq)]
pub enum AddressView {
    Opaque {
        address: Address,
    },
    Visible {
        address: Address,
        index: AddressIndex,
        wallet_id: WalletId,
    },
}

impl AddressView {
    pub fn address(&self) -> Address {
        match self {
            AddressView::Opaque { address } => *address,
            AddressView::Visible { address, .. } => *address,
        }
    }
}
