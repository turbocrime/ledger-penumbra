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

use crate::ParserError;

pub fn encode_varint(mut value: u64, buf: &mut [u8]) -> Result<usize, ParserError> {
    if buf.is_empty() {
        return Err(ParserError::InvalidLength);
    }

    let mut pos = 0;
    let size = buf.len();
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        buf[pos] = byte;
        pos += 1;
        if pos > size {
            return Err(ParserError::InvalidLength);
        }
        if value == 0 {
            break;
        }
    }

    Ok(pos)
}

pub fn encode_proto_number(tag: u64, value: u64, buf: &mut [u8]) -> Result<usize, ParserError> {
    if buf.is_empty() {
        return Err(ParserError::InvalidLength);
    }

    let mut len = encode_varint(tag << 3, buf)?;
    if len >= buf.len() {
        return Err(ParserError::InvalidLength);
    }

    len += encode_varint(value, &mut buf[len..])?;
    if len > buf.len() {
        return Err(ParserError::InvalidLength);
    }

    Ok(len)
}

pub fn encode_proto_field(
    tag: u64,
    wire_type: u64,
    size: usize,
    output: &mut [u8],
) -> Result<usize, ParserError> {
    if output.is_empty() {
        return Err(ParserError::InvalidLength);
    }

    let tag_and_type = (tag << 3) | wire_type;
    let mut len = encode_varint(tag_and_type, output)?;

    let remaining_buf = &mut output[len..];
    let varint_len = encode_varint(size as u64, remaining_buf)?;
    len += varint_len;

    if len > output.len() {
        return Err(ParserError::InvalidLength);
    }

    Ok(len)
}

pub fn encode_and_update_proto_field(
    state: &mut blake2b_simd::State,
    tag: u64,
    wire_type: u64,
    input: &[u8],
    size: usize,
) -> Result<(), ParserError> {
    if input.is_empty() || size > input.len() {
        return Err(ParserError::InvalidLength);
    }
    let mut proto_buf = [0u8; 20];
    let len = encode_proto_field(tag, wire_type, size, &mut proto_buf)?;

    state.update(&proto_buf[..len]);
    state.update(&input[..size]);
    Ok(())
}

pub fn encode_and_update_proto_number(
    state: &mut blake2b_simd::State,
    tag: u64,
    value: u64,
) -> Result<(), ParserError> {
    let mut proto_buf = [0u8; 20];
    let len = encode_proto_number(tag, value, &mut proto_buf)?;

    state.update(&proto_buf[..len]);
    Ok(())
}
