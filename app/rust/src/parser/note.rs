/*******************************************************************************
a   (c) 2024 Zondax GmbH
*
*  Licensed under the Apache License, Version 2.0 (the "License");
*  you may not use this file except in compliance with the License.
*  You may obtain a copy of the License at
*
*      http://www.apache.org/licenses/LICENSE-2.0
*
*  Unless required by applicable law or agreed to in writing, software
*  distributed under the License is distributed on an "AS IS" BASIS,
*  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*  See the License for the specific language governing permissions and
*  limitations under the License.
********************************************************************************/

use crate::address::Address;
use crate::keys::ka;
use crate::keys::ovk::Ovk;
use crate::parser::{
    address::AddressC,
    bytes::BytesC,
    commitment::{Commitment, StateCommitment},
    note_payload::NotePayload,
    rseed::Rseed,
    symmetric::PayloadKey,
    symmetric::{OutgoingCipherKey, OvkWrappedKey, PayloadKind, OVK_WRAPPED_LEN_BYTES},
    value::{Value, ValueC},
};
use crate::protobuf_h::shielded_pool_pb::{
    penumbra_core_component_shielded_pool_v1_NoteCiphertext_inner_tag, PB_LTYPE_UVARINT,
};
use crate::utils::apdu_unwrap::ApduPanic;
use crate::utils::protobuf::encode_proto_field;
use crate::ParserError;
use decaf377::{Element, Encoding, Fq, Fr};

pub const NOTE_LEN_BYTES: usize = 160;
pub const NOTE_CIPHERTEXT_BYTES: usize = 176;

#[derive(Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct NoteCiphertext(pub [u8; NOTE_CIPHERTEXT_BYTES]);

pub struct Note {
    /// The typed value recorded by this note.
    value: Value,
    /// A uniformly random 32-byte sequence used to derive an ephemeral secret key
    /// and note blinding factor.
    rseed: Rseed,
    /// The address controlling this note.
    address: Address,
    /// The s-component of the transmission key of the destination address.
    /// We store this separately to ensure that every `Note` is constructed
    /// with a valid transmission key (the `ka::Public` does not validate
    /// the curve point until it is used, since validation is not free).
    transmission_key_s: Fq,
}

#[repr(C)]
#[derive(Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct NoteC {
    pub has_value: bool,
    pub value: ValueC,
    pub rseed: BytesC,
    pub has_address: bool,
    pub address: AddressC,
}

impl TryFrom<NoteC> for Note {
    type Error = ParserError;

    fn try_from(note_c: NoteC) -> Result<Self, Self::Error> {
        let value = Value::try_from(note_c.value)?;
        let rseed = Rseed::try_from(note_c.rseed)?;
        let address = Address::try_from(note_c.address.inner.get_bytes()?)?;
        let transmission_key_s = Fq::from_bytes_checked(&address.transmission_key().0)
            .map_err(|_| ParserError::InvalidFvk)?;

        Ok(Note {
            value,
            rseed,
            address,
            transmission_key_s,
        })
    }
}

impl Note {
    pub fn from_parts(address: Address, value: Value, rseed: Rseed) -> Result<Self, ParserError> {
        Ok(Note {
            value,
            rseed,
            address,
            transmission_key_s: Fq::from_bytes_checked(&address.transmission_key().0)
                .map_err(|_| ParserError::InvalidFvk)?,
        })
    }

    pub fn commit(&self) -> Result<StateCommitment, ParserError> {
        let commit = poseidon377::hash_6(
            &Self::notecommit_domain_sep(),
            (
                self.note_blinding()?,
                self.value.amount.into(),
                self.value.asset_id.0,
                self.diversified_generator()?.vartime_compress_to_field(),
                self.transmission_key_s,
                self.clue_key()?,
            ),
        );
        Ok(StateCommitment(commit))
    }

    /// Generate encrypted outgoing cipher key for use with this note.
    pub fn encrypt_key(&self, ovk: &Ovk, cv: Commitment) -> Result<OvkWrappedKey, ParserError> {
        let esk = self.ephemeral_secret_key()?;
        let epk = esk.diversified_public(&self.diversified_generator()?);
        let ock = OutgoingCipherKey::derive(ovk, cv, &self.commit()?.0, &epk);

        let shared_secret = esk
            .key_agreement_with(self.transmission_key())
            .apdu_expect("key agreement succeeded");

        let mut encryption_result = [0u8; OVK_WRAPPED_LEN_BYTES];
        encryption_result[..shared_secret.0.len()].copy_from_slice(&shared_secret.0);
        ock.encrypt(&mut encryption_result, PayloadKind::Note)
            .map_err(|_| ParserError::UnexpectedError)?;

        Ok(OvkWrappedKey(
            encryption_result[..OVK_WRAPPED_LEN_BYTES]
                .try_into()
                .apdu_expect("OVK encryption result fits in ciphertext len"),
        ))
    }

    /// Encrypt a note, returning its ciphertext.
    pub fn encrypt(&self) -> Result<NoteCiphertext, ParserError> {
        let esk = self.ephemeral_secret_key()?;
        let epk = esk.diversified_public(&self.diversified_generator()?);
        let shared_secret = esk
            .key_agreement_with(self.transmission_key())
            .apdu_expect("key agreement succeeded");

        let key = PayloadKey::derive(&shared_secret, &epk);

        let mut encryption_result = [0u8; NOTE_CIPHERTEXT_BYTES];
        let note_plaintext: [u8; NOTE_LEN_BYTES] = self.try_into()?;
        encryption_result[..NOTE_LEN_BYTES].copy_from_slice(&note_plaintext);

        key.encrypt(&mut encryption_result, PayloadKind::Note, NOTE_LEN_BYTES)
            .map_err(|_| ParserError::UnexpectedError)?;

        let ciphertext: [u8; NOTE_CIPHERTEXT_BYTES] = encryption_result;

        Ok(NoteCiphertext(ciphertext))
    }

    pub fn payload(&self) -> Result<NotePayload, ParserError> {
        Ok(NotePayload {
            note_commitment: self.commit()?,
            ephemeral_key: self.ephemeral_public_key()?,
            encrypted_note: self.encrypt()?,
        })
    }

    pub fn note_blinding(&self) -> Result<Fq, ParserError> {
        let rseed = self.rseed.derive_note_blinding()?;
        Ok(rseed)
    }

    pub fn diversified_generator(&self) -> Result<Element, ParserError> {
        let diversifier_generator = self.address.diversifier().diversified_generator();
        Ok(diversifier_generator)
    }

    pub fn transmission_key(&self) -> &ka::Public {
        self.address.transmission_key()
    }

    pub fn key_agreement_with(
        &self,
        esk: &Fr,
        other: &ka::Public,
    ) -> Result<Encoding, ParserError> {
        let pk = decaf377::Encoding(other.0)
            .vartime_decompress()
            .map_err(|_| ParserError::InvalidFvk)?;

        Ok((esk * pk).vartime_compress())
    }

    pub fn clue_key(&self) -> Result<Fq, ParserError> {
        let clue_key = self.address.clue_key();
        Ok(Fq::from_le_bytes_mod_order(&clue_key.0[..]))
    }

    pub fn notecommit_domain_sep() -> Fq {
        Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.notecommit").as_bytes())
    }

    pub fn ephemeral_secret_key(&self) -> Result<ka::Secret, ParserError> {
        self.rseed.derive_esk()
    }

    pub fn ephemeral_public_key(&self) -> Result<ka::Public, ParserError> {
        Ok(self
            .ephemeral_secret_key()
            .map_err(|_| ParserError::UnexpectedError)?
            .diversified_public(&self.diversified_generator()?))
    }
}

impl NoteCiphertext {
    pub const PROTO_LEN: usize = NOTE_CIPHERTEXT_BYTES + 3;

    pub fn to_proto(&self) -> Result<[u8; Self::PROTO_LEN], ParserError> {
        let mut proto = [0u8; Self::PROTO_LEN];

        let len = encode_proto_field(
            penumbra_core_component_shielded_pool_v1_NoteCiphertext_inner_tag as u64,
            PB_LTYPE_UVARINT as u64,
            self.0.len(),
            &mut proto,
        )?;

        proto[len..].copy_from_slice(&self.0);
        Ok(proto)
    }
}

impl TryFrom<&Note> for [u8; NOTE_LEN_BYTES] {
    type Error = ParserError;

    fn try_from(note: &Note) -> Result<[u8; NOTE_LEN_BYTES], Self::Error> {
        let mut bytes = [0u8; NOTE_LEN_BYTES];
        bytes[0..80].copy_from_slice(&note.address.to_bytes()?);
        bytes[80..96].copy_from_slice(&note.value.amount.to_le_bytes());
        bytes[96..128].copy_from_slice(&note.value.asset_id.to_bytes());
        bytes[128..160].copy_from_slice(&note.rseed.to_bytes()?);
        Ok(bytes)
    }
}
