/*******************************************************************************
*   (c) 2018 - 2024 Zondax AG
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
#![no_std]
#![no_builtins]
#![allow(dead_code)]
#![deny(unused_crate_dependencies)]

extern crate no_std_compat as std;

#[cfg(test)]
extern crate hex_literal;

use poseidon377 as _;
use educe as _;
use arrayref as _;

pub(crate) mod address;
mod bolos;
pub mod constants;
pub mod ffi;
pub(crate) mod keys;
pub mod network;
pub mod parser;
mod utils;
pub mod wallet_id;

pub use parser::{FromBytes, ParserError, ViewError};
pub(crate) use utils::prf::{expand_fq, expand_fr};

fn debug(_msg: &str) {}

#[cfg(all(not(test), not(feature = "clippy"), not(feature = "fuzzing")))]
use core::panic::PanicInfo;

#[cfg(all(not(test), not(feature = "clippy"), not(feature = "fuzzing")))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

extern "C" {
    fn check_app_canary();
    fn pic(link_address: u32) -> u32;
    fn app_mode_expert() -> u8;
    fn zemu_log_stack(s: *const u8);
}

pub(crate) fn canary() {
    #[cfg(not(any(test, fuzzing)))]
    unsafe {
        check_app_canary();
    }
}

#[cfg(not(any(test, fuzzing)))]
pub fn is_expert_mode() -> bool {
    unsafe { app_mode_expert() > 0 }
}

#[cfg(any(test, feature = "fuzzing"))]
pub fn is_expert_mode() -> bool {
    true
}

pub fn zlog(_msg: &str) {
    #[cfg(not(any(test, fuzzing)))]
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
