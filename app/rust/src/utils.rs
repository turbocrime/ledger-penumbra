mod apdu_unwrap;
pub mod prf;
pub mod protobuf;

use crate::ParserError;
#[cfg(test)]
use std::vec::Vec;

pub fn varint(input: &[u8]) -> Result<(&[u8], u64), ParserError> {
    let mut value = 0u64;
    let mut shift = 0u32;

    for (i, &byte) in input.iter().enumerate() {
        let byte_val = byte as u64;
        value |= (byte_val & 0x7F) << shift;
        if byte_val & 0x80 == 0 {
            return Ok((&input[i + 1..], value));
        }
        shift += 7;
        if shift >= 70 {
            // Varints are a maximum of 10 bytes in length, and 70 bits in shift
            break;
        }
    }

    Err(nom::Err::Error(nom::error::ParseError::from_error_kind(
        input,
        nom::error::ErrorKind::Eof,
    ))
    .into())
}

pub fn read_string(input: &[u8]) -> Result<(&[u8], &str), ParserError> {
    // First, read the length of the string using varint
    let (remaining, length) = varint(input)?;

    // Check if we have enough bytes for the string
    if remaining.len() < length as usize {
        return Err(nom::Err::Error(nom::error::ParseError::from_error_kind(
            input,
            nom::error::ErrorKind::Eof,
        ))
        .into());
    }

    // Split the input at the string boundary
    let (string_bytes, rest) = remaining.split_at(length as usize);

    // Attempt to convert bytes to str
    match core::str::from_utf8(string_bytes) {
        Ok(s) => Ok((rest, s)),
        Err(_) => Err(nom::Err::Error(nom::error::ParseError::from_error_kind(
            input,
            nom::error::ErrorKind::Char,
        ))
        .into()),
    }
}

pub fn read_bytes(input: &[u8]) -> Result<(&[u8], &[u8]), ParserError> {
    // First, read the length of the bytes using varint
    let (remaining, length) = varint(input)?;

    // Check if we have enough bytes
    if remaining.len() < length as usize {
        return Err(nom::Err::Error(nom::error::ParseError::from_error_kind(
            input,
            nom::error::ErrorKind::Eof,
        ))
        .into());
    }

    // Split the input at the bytes boundary
    let (bytes, rest) = remaining.split_at(length as usize);

    Ok((rest, bytes))
}

pub fn read_fixed_bytes<'a, const SIZE: usize>(
    input: &'a [u8],
) -> Result<(&'a [u8], &'a [u8; SIZE]), ParserError> {
    let (rest, bytes) = read_bytes(input)?;

    // Check if we have enough bytes
    if bytes.len() != SIZE {
        return Err(nom::Err::Error(nom::error::ParseError::from_error_kind(
            input,
            nom::error::ErrorKind::Eof,
        ))
        .into());
    }

    // Convert the bytes slice to a fixed-size array
    let bytes: &'a [u8; SIZE] = bytes.try_into().map_err(|_| {
        nom::Err::Error(nom::error::ParseError::from_error_kind(
            input,
            nom::error::ErrorKind::LengthValue,
        ))
    })?;

    Ok((rest, bytes))
}

#[cfg(test)]
// Helper function to encode a u64 value as a varint
pub fn encode_varint(value: u64) -> Vec<u8> {
    let mut buffer = Vec::new();
    let mut temp = value;
    while temp >= 0x80 {
        buffer.push((temp as u8) | 0x80);
        temp >>= 7;
    }
    buffer.push(temp as u8);
    buffer
}

#[cfg(test)]
pub fn encode_bytes<B: AsRef<[u8]>>(input: B) -> Vec<u8> {
    let input = input.as_ref();
    // Encode as Protobuf (field number 1, type 2 for bytes, length)
    let mut encoded_data = Vec::new();
    encoded_data.extend_from_slice(&[0x0A]); // field number 1, wire type 2
    let len = encode_varint(input.len() as _);
    encoded_data.extend_from_slice(len.as_slice()); // length of the bytes field
    encoded_data.extend_from_slice(input);

    encoded_data
}
