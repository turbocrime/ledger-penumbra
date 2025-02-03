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

//! Rust interfaces to Ledger SDK APIs.
#[cfg(test)]
use getrandom::getrandom;

use rand::{CryptoRng, RngCore};

#[cfg(all(
    not(test),
    not(feature = "clippy"),
    not(feature = "fuzzing"),
    not(feature = "cpp_tests")
))]
extern "C" {
    fn check_app_canary();
    fn pic(link_address: u32) -> u32;
    fn app_mode_expert() -> u8;
    fn zemu_log_stack(s: *const u8);
    fn io_heartbeat();
}

extern "C" {
    fn cx_rng(buffer: *mut u8, len: u32);
}

pub struct Trng;

impl RngCore for Trng {
    fn next_u32(&mut self) -> u32 {
        let mut out = [0; 4];
        self.fill_bytes(&mut out);
        u32::from_le_bytes(out)
    }

    fn next_u64(&mut self) -> u64 {
        let mut out = [0; 8];
        self.fill_bytes(&mut out);
        u64::from_le_bytes(out)
    }

    #[cfg(not(test))]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        zlog("fill_bytes\x00");

        unsafe {
            cx_rng(dest.as_mut_ptr(), dest.len() as u32);
        }
    }

    #[cfg(test)]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        let _ = getrandom(dest);
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl CryptoRng for Trng {}

pub(crate) fn canary() {
    #[cfg(all(
        not(test),
        not(feature = "clippy"),
        not(feature = "fuzzing"),
        not(feature = "cpp_tests")
    ))]
    unsafe {
        check_app_canary();
    }
}

pub fn is_expert_mode() -> bool {
    cfg_if::cfg_if! {
        if #[cfg(all(not(test), not(feature = "clippy"), not(feature = "fuzzing"), not(feature = "cpp_tests")))] {
            unsafe {
                app_mode_expert() > 0
            }
        } else {
            true
        }
    }
}

pub fn zlog(_msg: &str) {
    #[cfg(all(
        not(test),
        not(feature = "clippy"),
        not(feature = "fuzzing"),
        not(feature = "cpp_tests")
    ))]
    unsafe {
        zemu_log_stack(_msg.as_bytes().as_ptr());
    }
}

#[macro_export]
macro_rules! check_canary {
    () => {
        use canary;
        canary();
    };
}

pub fn pic_addr(addr: u32) -> u32 {
    cfg_if::cfg_if! {
        if #[cfg(all(not(test), not(feature = "clippy"), not(feature = "fuzzing"), not(feature = "cpp_tests")))] {
        unsafe {
            pic(addr)
        }
        } else {
            addr
        }
    }
}

// Lets the device breath between computations
pub(crate) fn heartbeat() {
    #[cfg(all(
        not(test),
        not(feature = "clippy"),
        not(feature = "fuzzing"),
        not(feature = "cpp_tests")
    ))]
    unsafe {
        io_heartbeat();
    }
}
