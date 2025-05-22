use decaf377::{Fq, Fr};

use crate::{
    address::{Address, AddressIndex},
    utils::prf,
    ParserError,
};

use super::{
    detection_key::DetectionKey,
    dk::{Diversifier, DiversifierKey},
    ka::Secret,
    spend_key::SpendKeyBytes,
};

/// An incoming viewing key
#[derive(Clone, PartialEq, Eq)]
pub struct Ivk {
    pub(super) ivk: Secret,
    pub(super) dk: DiversifierKey,
}

impl Ivk {
    pub const IVK_DOMAIN_SEP: [u8; 19] = *b"penumbra.derive.ivk";

    /// Derive an incoming viewing key from a spend key.
    /// based on the reference implementation at:
    /// https://github.com/penumbra-zone/penumbra/blob/main/crates/core/keys/src/keys/fvk.rs#L92
    pub fn derive_from(spend_key: &SpendKeyBytes) -> Result<Self, ParserError> {
        let dk = spend_key.diversifier_key()?;
        let ak = spend_key.verification_key()?;
        let nk = spend_key.nullifier_key()?;

        let ak_s = Fq::from_bytes_checked(ak.as_ref()).map_err(|_| ParserError::InvalidLength)?;
        let domain_sep = Fq::from_le_bytes_mod_order(&Self::IVK_DOMAIN_SEP);

        // now compute the ivk_mod_q using poseidon377::hash function
        // with the result compute the ivk secret component:
        // current poseidon377 hash function uses std and allocation.
        let ivk_mod_q = poseidon377::hash_2(&domain_sep, (nk.0, ak_s));

        let ivk = Secret::new_from_field(Fr::from_le_bytes_mod_order(&ivk_mod_q.to_bytes()));
        Ok(Self { ivk, dk })
    }

    pub fn payment_address(
        &self,
        index: AddressIndex,
    ) -> Result<(Address, DetectionKey), ParserError> {
        let d = self.dk.diversifier_for_index(&index);

        let g_d = d.diversified_generator();
        let pk_d = self.ivk.diversified_public(&g_d);

        // PenumbraExpndFMD
        let expanded = prf::expand(DetectionKey::LABEL, &self.ivk.to_bytes(), d.as_ref())?;

        let dtk_d = DetectionKey::from_field(Fr::from_le_bytes_mod_order(&expanded));
        let ck_d = dtk_d.clue_key();

        let address = Address::from_components(d, pk_d, ck_d)?;
        Ok((address, dtk_d))
    }

    /// Check whether this address is viewable by this incoming viewing key.
    pub fn views_address(&self, address: &Address) -> bool {
        self.ivk.diversified_public(address.diversified_generator()) == *address.transmission_key()
    }

    /// Returns the index used to create the given diversifier (if it was
    /// created using this incoming viewing key)
    pub fn index_for_diversifier(&self, diversifier: &Diversifier) -> AddressIndex {
        self.dk.index_for_diversifier(diversifier)
    }

    /// Returns the index of the given address, if the address is viewed by this
    /// viewing key; otherwise, returns `None`.
    pub(super) fn address_index(&self, address: &Address) -> Option<AddressIndex> {
        if self.views_address(address) {
            Some(self.index_for_diversifier(address.diversifier()))
        } else {
            None
        }
    }
}

impl TryFrom<&SpendKeyBytes> for Ivk {
    type Error = ParserError;

    fn try_from(value: &SpendKeyBytes) -> Result<Self, Self::Error> {
        Self::derive_from(value)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const SPEND_KEY: &str = "ff726c71bcec76abc6a88cba71df655b28de6580edbd33c7415fdfded2e422e7";
    const SECRECT_IVK: &str = "50568b11d5668f53629af6531072e8666b07733db04d8953277e0df4a5382a01";
    const DIVERSIFIER: &str = "9d2107be5bfa0c07a7f870e216f185d9";

    #[test]
    fn test_derive_ivk() {
        let key_bytes = hex::decode(SPEND_KEY).unwrap();
        let key_bytes: [u8; 32] = key_bytes.as_slice().try_into().unwrap();
        let spk = SpendKeyBytes::from(key_bytes);
        let fvk = spk.fvk().unwrap();
        let ivk = fvk.ivk();
        let secrect = hex::encode(ivk.ivk.to_bytes());
        let div_s = hex::encode(ivk.dk.0);

        assert_eq!(secrect, SECRECT_IVK);
        assert_eq!(div_s, DIVERSIFIER);
    }
}
