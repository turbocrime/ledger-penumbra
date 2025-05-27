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
// Only enable the unused_crate_dependencies lint when not building for ARM bare metal (Ledger targets)
// This avoids false positives with compiler_builtins. See: https://github.com/rust-lang/rust/issues/106665
#![cfg_attr(not(all(target_arch = "arm", target_os = "none")), warn(unused_crate_dependencies))]

extern crate no_std_compat as std;

use ethnum as _;
use poseidon377 as _;

pub(crate) mod address;
mod bolos;
pub mod constants;
pub mod ffi;
pub(crate) mod keys;
pub mod network;
pub mod parser;
pub mod protobuf_h;
mod utils;
pub mod wallet_id;
pub mod zxerror;

pub use parser::{FromBytes, ParserError, ViewError};
pub(crate) use utils::prf::{expand_fq, expand_fr};

pub(crate) use bolos::*;

fn debug(_msg: &str) {}

// for cpp_tests we need to define the panic handler
// the remaining features does not need as dev-dependencies
// are used and their include a handler from std
#[cfg(all(not(test), not(feature = "clippy"), not(feature = "fuzzing"),))]
use core::panic::PanicInfo;

#[cfg(all(not(test), not(feature = "clippy"), not(feature = "fuzzing"),))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
