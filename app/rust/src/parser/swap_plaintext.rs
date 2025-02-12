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

use crate::address::Address;
use crate::constants::{SWAP_CIPHERTEXT_BYTES, SWAP_LEN_BYTES};
use crate::keys::ovk::Ovk;
use crate::parser::{
    address::AddressC,
    amount::{Amount, AmountC},
    bytes::BytesC,
    commitment::StateCommitment,
    fee::{Fee, FeeC},
    rseed::Rseed,
    swap_ciphertext::SwapCiphertext,
    swap_payload::SwapPayload,
    symmetric::PayloadKey,
    trading_pair::TradingPair,
    trading_pair::TradingPairC,
};
use crate::ParserError;
use decaf377::Fq;
use poseidon377::{hash_4, hash_7};

#[derive(Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct SwapPlaintext {
    pub trading_pair: TradingPair,
    pub delta_1_i: Amount,
    pub delta_2_i: Amount,
    pub claim_fee: Fee,
    pub claim_address: Address,
    pub rseed: Rseed,
}
#[repr(C)]
#[derive(Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct SwapPlaintextC {
    pub has_trading_pair: bool,
    pub trading_pair: TradingPairC,
    pub has_delta_1_i: bool,
    pub delta_1_i: AmountC,
    pub has_delta_2_i: bool,
    pub delta_2_i: AmountC,
    pub has_claim_fee: bool,
    pub claim_fee: FeeC,
    pub has_claim_address: bool,
    pub claim_address: AddressC,
    pub rseed: BytesC,
}

impl TryFrom<SwapPlaintextC> for SwapPlaintext {
    type Error = ParserError;

    fn try_from(value: SwapPlaintextC) -> Result<Self, Self::Error> {
        let trading_pair = TradingPair::try_from(value.trading_pair)?;
        let claim_fee = Fee::try_from(value.claim_fee)?;
        let claim_address = Address::try_from(value.claim_address.inner.get_bytes()?)?;
        let rseed = Rseed::try_from(value.rseed)?;

        Ok(SwapPlaintext {
            trading_pair,
            delta_1_i: Amount::try_from(value.delta_1_i)?,
            delta_2_i: Amount::try_from(value.delta_2_i)?,
            claim_fee,
            claim_address,
            rseed,
        })
    }
}

impl SwapPlaintext {
    pub fn encrypt(&self, ovk: &Ovk) -> Result<SwapPayload, ParserError> {
        let commitment = self.swap_commitment()?;
        let key = PayloadKey::derive_swap(ovk, commitment.clone());
        let swap_plaintext: [u8; SWAP_LEN_BYTES] = self.to_bytes()?;

        let mut encryption_result = [0u8; SWAP_CIPHERTEXT_BYTES];
        encryption_result[..SWAP_LEN_BYTES].copy_from_slice(&swap_plaintext);

        key.encrypt_swap(&mut encryption_result, SWAP_LEN_BYTES)?;

        let ciphertext: [u8; SWAP_CIPHERTEXT_BYTES] = encryption_result;

        Ok(SwapPayload {
            encrypted_swap: SwapCiphertext(ciphertext),
            commitment,
        })
    }

    // Constructs the unique asset ID for a swap as a poseidon hash of the input data for the swap.
    //
    // https://protocol.penumbra.zone/main/zswap/swap.html#swap-actions
    pub fn swap_commitment(&self) -> Result<StateCommitment, ParserError> {
        let inner = hash_7(
            &Self::swap_domain_sep(),
            (
                Fq::from_le_bytes_mod_order(self.rseed.to_bytes()?.as_ref()),
                self.claim_fee.0.amount.into(),
                self.claim_fee.0.asset_id.0,
                self.claim_address
                    .diversified_generator()
                    .vartime_compress_to_field(),
                *self.claim_address.transmission_key_s(),
                Fq::from_le_bytes_mod_order(&self.claim_address.clue_key().0[..]),
                hash_4(
                    &Self::swap_domain_sep(),
                    (
                        self.trading_pair.asset_1().0,
                        self.trading_pair.asset_2().0,
                        self.delta_1_i.into(),
                        self.delta_2_i.into(),
                    ),
                ),
            ),
        );

        Ok(StateCommitment(inner))
    }

    fn swap_domain_sep() -> Fq {
        Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.swap").as_bytes())
    }

    fn to_bytes(&self) -> Result<[u8; SWAP_LEN_BYTES], ParserError> {
        let mut bytes = [0u8; SWAP_LEN_BYTES];
        let mut offset = 0;

        // Write trading pair bytes
        let trading_pair_bytes = self.trading_pair.to_bytes()?;
        bytes[offset..offset + trading_pair_bytes.len()].copy_from_slice(&trading_pair_bytes);
        offset += trading_pair_bytes.len();

        // Write delta_1_i bytes
        let delta_1_bytes = self.delta_1_i.to_le_bytes();
        bytes[offset..offset + delta_1_bytes.len()].copy_from_slice(&delta_1_bytes);
        offset += delta_1_bytes.len();

        // Write delta_2_i bytes
        let delta_2_bytes = self.delta_2_i.to_le_bytes();
        bytes[offset..offset + delta_2_bytes.len()].copy_from_slice(&delta_2_bytes);
        offset += delta_2_bytes.len();

        // Write claim fee bytes
        let fee_bytes = self.claim_fee.to_bytes()?;
        bytes[offset..offset + fee_bytes.len()].copy_from_slice(&fee_bytes);
        offset += fee_bytes.len();

        // Write claim address bytes
        let addr_bytes = self.claim_address.to_bytes()?;
        bytes[offset..offset + addr_bytes.len()].copy_from_slice(&addr_bytes);
        offset += addr_bytes.len();

        // Write rseed bytes
        let rseed_bytes = self.rseed.to_bytes()?;
        bytes[offset..offset + rseed_bytes.len()].copy_from_slice(&rseed_bytes);
        Ok(bytes)
    }
}
