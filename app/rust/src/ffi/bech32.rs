use bech32::{Bech32m, ByteIterExt, Fe32IterExt, Hrp};

/// Encodes data using the Bech32m format.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers. The caller must ensure that:
/// - `hrp_ptr` points to a valid memory location with at least `hrp_len` bytes.
/// - `data_ptr` points to a valid memory location with at least `data_len` bytes.
/// - `output_ptr` points to a valid memory location with at least `output_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_bech32_encode(
    hrp_ptr: *const u8,
    hrp_len: usize,
    data_ptr: *const u8,
    data_len: usize,
    output_ptr: *mut u8,
    output_len: usize,
) -> i32 {
    crate::zlog("rs_bech32_encode\x00");

    let hrp_slice = std::slice::from_raw_parts(hrp_ptr, hrp_len);
    let data_slice = std::slice::from_raw_parts(data_ptr, data_len);
    let output_slice = std::slice::from_raw_parts_mut(output_ptr, output_len);

    // Parse HRP
    let hrp_str = match std::str::from_utf8(hrp_slice) {
        Ok(s) => s,
        Err(_) => return -1,
    };

    match bech32_encode(hrp_str, data_slice, output_slice) {
        Ok(written) => written as i32,
        Err(e) => e,
    }
}

pub fn bech32_encode(hrp: &str, data: &[u8], output: &mut [u8]) -> Result<usize, i32> {
    let hrp = Hrp::parse(hrp).map_err(|_| -2)?; // Invalid HRP

    let chars = data
        .iter()
        .copied()
        .bytes_to_fes()
        .with_checksum::<Bech32m>(&hrp)
        .chars();

    // Copy characters to the output buffer
    let mut written = 0;
    for c in chars {
        if written >= output.len() {
            return Err(-3);
        }
        output[written] = c as u8;
        written += 1;
    }

    Ok(written)
}
