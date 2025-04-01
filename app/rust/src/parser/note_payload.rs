/*******************************************************************************
*   (c) 2024 Zondax GmbH
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

use crate::keys::ka;
use crate::parser::commitment::StateCommitment;
use crate::parser::note::NoteCiphertext;
use crate::parser::note::NOTE_CIPHERTEXT_BYTES;
use crate::protobuf_h::shielded_pool_pb::{
    penumbra_core_component_shielded_pool_v1_NotePayload_encrypted_note_tag,
    penumbra_core_component_shielded_pool_v1_NotePayload_ephemeral_key_tag,
    penumbra_core_component_shielded_pool_v1_NotePayload_note_commitment_tag, PB_LTYPE_UVARINT,
};
use crate::utils::protobuf::encode_proto_field;
use crate::ParserError;

#[derive(Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct NotePayload {
    pub note_commitment: StateCommitment,
    pub ephemeral_key: ka::Public,
    pub encrypted_note: NoteCiphertext,
}

impl NotePayload {
    pub const LEN: usize = 32 + 32 + NOTE_CIPHERTEXT_BYTES;
    pub const PROTO_LEN: usize = Self::LEN + 12;

    pub fn to_proto(&self) -> Result<[u8; Self::PROTO_LEN], ParserError> {
        let mut proto = [0u8; Self::PROTO_LEN];
        let mut offset = 0;

        // Encode note_commitment
        let note_commitment_bytes = self.note_commitment.to_proto()?;
        offset += encode_proto_field(
            penumbra_core_component_shielded_pool_v1_NotePayload_note_commitment_tag as u64,
            PB_LTYPE_UVARINT as u64,
            note_commitment_bytes.len(),
            &mut proto[offset..offset + 2],
        )?;
        proto[offset..offset + note_commitment_bytes.len()].copy_from_slice(&note_commitment_bytes);
        offset += note_commitment_bytes.len();

        // Encode ephemeral_key
        offset += encode_proto_field(
            penumbra_core_component_shielded_pool_v1_NotePayload_ephemeral_key_tag as u64,
            PB_LTYPE_UVARINT as u64,
            self.ephemeral_key.0.len(),
            &mut proto[offset..offset + 2],
        )?;
        proto[offset..offset + self.ephemeral_key.0.len()].copy_from_slice(&self.ephemeral_key.0);
        offset += self.ephemeral_key.0.len();

        // Encode encrypted_note
        let encrypted_note_bytes = self.encrypted_note.to_proto()?;
        offset += encode_proto_field(
            penumbra_core_component_shielded_pool_v1_NotePayload_encrypted_note_tag as u64,
            PB_LTYPE_UVARINT as u64,
            encrypted_note_bytes.len(),
            &mut proto[offset..offset + 3],
        )?;
        proto[offset..offset + encrypted_note_bytes.len()].copy_from_slice(&encrypted_note_bytes);
        offset += encrypted_note_bytes.len();

        if offset != Self::PROTO_LEN {
            return Err(ParserError::InvalidLength);
        }

        Ok(proto)
    }
}
