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

use arrayvec::CapacityError;
use nom::error::ErrorKind;

#[repr(u32)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
// #[cfg_attr(test, derive(Debug))]
pub enum ParserError {
    Ok = 0,
    // Generic errors
    NoData,
    DisplayIdxOutOfRange,
    DisplayPageOutOfRange,
    UnexpectedError,
    // Required fields
    // Coin specific
    InvalidHashMode,
    InvalidSignature,
    InvalidPubkeyEncoding,
    InvalidAddressVersion,
    InvalidAddressLength,
    InvalidTypeId,
    InvalidCodec,
    InvalidThreshold,
    InvalidNetworkId,
    InvalidChainId,
    InvalidAsciiValue,
    InvalidTimestamp,
    InvalidStakingAmount,
    UnexpectedType,
    InvalidTransactionType,
    OperationOverflows,
    UnexpectedBufferEnd,
    UnexpectedNumberItems,
    UnexpectedField,
    ValueOutOfRange,
    InvalidAddress,
    InvalidPath,
    InvalidLength,
    TooManyOutputs,
    UnexpectedData,
    InvalidClueKey,
    InvalidTxKey,
    InvalidFq,
    InvalidDetectionKey,
    InvalidFvk,
    InvalidIvk,
    InvalidKeyLen,
    InvalidActionType,
    InvalidPrecision,
    PrecisionTooLarge,
    ClueCreationFailed,
}

impl From<ErrorKind> for ParserError {
    fn from(err: ErrorKind) -> Self {
        match err {
            ErrorKind::Eof => ParserError::UnexpectedBufferEnd,
            ErrorKind::Permutation => ParserError::UnexpectedType,
            ErrorKind::TooLarge => ParserError::ValueOutOfRange,
            ErrorKind::Tag => ParserError::InvalidTypeId,
            _ => ParserError::UnexpectedError,
        }
    }
}

impl<I> nom::error::ParseError<I> for ParserError {
    fn from_error_kind(_input: I, kind: ErrorKind) -> Self {
        Self::from(kind)
    }

    // We don't have enough memory resources to use here an array with the last
    // N errors to be used as a backtrace, so that, we just propagate here the latest
    // reported error
    fn append(_input: I, _kind: ErrorKind, other: Self) -> Self {
        other
    }
}
impl From<ParserError> for nom::Err<ParserError> {
    fn from(error: ParserError) -> Self {
        nom::Err::Error(error)
    }
}

impl From<CapacityError> for ParserError {
    fn from(_error: CapacityError) -> Self {
        ParserError::UnexpectedBufferEnd
    }
}

impl From<nom::Err<Self>> for ParserError {
    fn from(e: nom::Err<Self>) -> Self {
        match e {
            nom::Err::Error(e) => e,
            nom::Err::Failure(e) => e,
            nom::Err::Incomplete(_) => Self::UnexpectedBufferEnd,
        }
    }
}
