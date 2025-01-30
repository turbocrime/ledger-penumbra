use crate::{
    address::{address_view::AddressView, Address, AddressIndex},
    utils::prf,
    wallet_id::WalletId,
    ParserError,
};

use super::{
    detection_key::DetectionKey, dk::DiversifierKey, ka, nk::NullifierKey,
    spend_key::SpendKeyBytes, Ivk, Ovk,
};
use decaf377::{Fq, Fr};
use decaf377_rdsa::{SpendAuth, VerificationKey};

/// The root viewing capability for all data related to a given spend authority.
#[derive(Clone, PartialEq, Eq)]
pub struct FullViewingKey {
    ak: VerificationKey<SpendAuth>,
    nk: NullifierKey,
    ovk: Ovk,
    ivk: Ivk,
}

impl FullViewingKey {
    pub const ACCOUNT_ID_DOMAIN_SEP: &'static [u8] = b"Penumbra_HashFVK";

    pub(crate) fn derive_from(spk: &SpendKeyBytes) -> Result<Self, ParserError> {
        crate::zlog("FullViewingKey::derive_from\x00");
        let ak = spk.verification_key()?;
        let nk = spk.nullifier_key()?;
        let ovk = Ovk::derive_from(spk)?;
        let ivk = Ivk::derive_from(spk)?;
        Ok(Self { ak, nk, ovk, ivk })
    }

    /// Derive a shielded payment address with the given [`AddressIndex`].
    pub fn payment_address(
        &self,
        index: AddressIndex,
    ) -> Result<(Address, DetectionKey), ParserError> {
        self.ivk().payment_address(index)
    }

    /// Views the structure of the supplied address with this viewing key.
    pub fn view_address(&self, address: Address) -> Result<AddressView, ParserError> {
        if self.ivk().views_address(&address) {
            Ok(AddressView::Visible {
                index: self.ivk().index_for_diversifier(address.diversifier()),
                wallet_id: self.wallet_id()?,
                address,
            })
        } else {
            Ok(AddressView::Opaque { address })
        }
    }

    /// Returns the index of the given address, if the address is viewed by this
    /// viewing key; otherwise, returns `None`.
    pub fn address_index(&self, address: &Address) -> Option<AddressIndex> {
        self.ivk().address_index(address)
    }

    /// Construct a full viewing key from its components.
    pub fn from_components(
        ak: VerificationKey<SpendAuth>,
        nk: NullifierKey,
    ) -> Result<Self, ParserError> {
        crate::heartbeat();
        let ovk = {
            let hash_result = prf::expand(b"Penumbra_DeriOVK", &nk.0.to_bytes(), ak.as_ref())?;
            let mut ovk = [0; 32];
            ovk.copy_from_slice(&hash_result[0..32]);
            ovk
        };
        crate::heartbeat();

        let dk = {
            let hash_result = prf::expand(b"Penumbra_DerivDK", &nk.0.to_bytes(), ak.as_ref())?;
            let mut dk = [0; 16];
            dk.copy_from_slice(&hash_result[0..16]);
            dk
        };

        let ivk = {
            let ak_s = Fq::from_bytes_checked(ak.as_ref()).map_err(|_| ParserError::InvalidFq)?;
            let domain_sep = Fq::from_le_bytes_mod_order(&Ivk::IVK_DOMAIN_SEP);
            let ivk_mod_q = poseidon377::hash_2(&domain_sep, (nk.0, ak_s));
            ka::Secret::new_from_field(Fr::from_le_bytes_mod_order(&ivk_mod_q.to_bytes()))
        };
        crate::heartbeat();

        let dk = DiversifierKey(dk);
        let ovk = Ovk(ovk);
        let ivk = Ivk { ivk, dk };

        Ok(Self { ak, nk, ovk, ivk })
    }

    /// Returns the ivk viewing key for this full viewing key.
    pub fn ivk(&self) -> &Ivk {
        &self.ivk
    }

    /// Returns the outgoing viewing key for this full viewing key.
    pub fn outgoing(&self) -> &Ovk {
        &self.ovk
    }

    pub fn nullifier_key(&self) -> &NullifierKey {
        &self.nk
    }

    /// Returns the spend verification key contained in this full viewing key.
    pub fn spend_verification_key(&self) -> &VerificationKey<SpendAuth> {
        &self.ak
    }

    /// Hashes the full viewing key into an [`WalletId`].
    pub fn wallet_id(&self) -> Result<WalletId, ParserError> {
        let domain_sep = Fq::from_le_bytes_mod_order(Self::ACCOUNT_ID_DOMAIN_SEP);
        let hash_result = poseidon377::hash_2(
            &domain_sep,
            (
                self.nk.0,
                Fq::from_le_bytes_mod_order(&self.ak.to_bytes()[..]),
            ),
        );
        let hash = hash_result.to_bytes()[..32]
            .try_into()
            .map_err(|_| ParserError::InvalidAddress)?;
        Ok(WalletId(hash))
    }

    /// Concatenate VerificationKey and NullifierKey into passed buffer
    /// Returns OK on success.
    pub fn to_bytes_into(&self, out: &mut [u8]) -> Result<(), ParserError> {
        // vk and nk are 32 bytes each
        if out.len() < (crate::constants::KEY_LEN * 2) {
            return Err(ParserError::InvalidLength);
        }

        out[0..32].copy_from_slice(&self.ak.to_bytes());
        out[32..64].copy_from_slice(&self.nk.0.to_bytes());

        Ok(())
    }
}

impl TryFrom<&SpendKeyBytes> for FullViewingKey {
    type Error = ParserError;

    fn try_from(spend_key: &SpendKeyBytes) -> Result<FullViewingKey, Self::Error> {
        FullViewingKey::derive_from(spend_key)
    }
}
