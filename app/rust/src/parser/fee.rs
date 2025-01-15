use crate::constants::{AMOUNT_LEN_BYTES, ID_LEN_BYTES};
use crate::parser::commitment::Commitment;
use crate::parser::id::Id;
use crate::parser::value::Sign;
use crate::parser::value::{Balance, Imbalance, Value, ValueC};
use crate::ParserError;
use decaf377::Fq;
use decaf377::Fr;
// The staking token asset ID (upenumbra)
// Bech32m: passet1984fctenw8m2fpl8a9wzguzp7j34d7vravryuhft808nyt9fdggqxmanqm
pub const STAKING_TOKEN_ASSET_ID_BYTES: [u8; 32] = [
    0x29, 0xea, 0x9c, 0x2f, 0x33, 0x71, 0xf6, 0xa4, 0x87, 0xe7, 0xe9, 0x5c, 0x24, 0x70, 0x41, 0xf4,
    0xa3, 0x56, 0xf9, 0x83, 0xeb, 0x06, 0x4e, 0x5d, 0x2b, 0x3b, 0xcf, 0x32, 0x2c, 0xa9, 0x6a, 0x10,
];

#[derive(Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct Fee(pub Value);

#[repr(C)]
#[derive(Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct FeeC(pub ValueC);

impl TryFrom<FeeC> for Fee {
    type Error = ParserError;

    fn try_from(value: FeeC) -> Result<Self, Self::Error> {
        if value.0.has_asset_id {
            Ok(Fee(Value::try_from(value.0)?))
        } else {
            // If conversion fails, create a new Value with the amount and staking token asset ID
            Ok(Fee(Value {
                amount: value.0.amount.try_into()?,
                asset_id: Id(Fq::from_le_bytes_mod_order(&STAKING_TOKEN_ASSET_ID_BYTES)),
            }))
        }
    }
}

impl FeeC {
    pub fn to_value_c(&self) -> ValueC {
        self.0.clone()
    }
}

impl Fee {
    pub const LEN: usize = AMOUNT_LEN_BYTES + ID_LEN_BYTES;

    pub fn commit(&self, blinding: Fr) -> Result<Commitment, ParserError> {
        let mut balance = Balance::new();
        balance.add(Imbalance {
            value: self.0.clone(),
            sign: Sign::Required,
        })?;
        balance.commit(blinding)
    }

    pub fn to_bytes(&self) -> Result<[u8; Self::LEN], ParserError> {
        self.0.to_bytes()
    }
}
