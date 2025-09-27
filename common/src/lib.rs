#![cfg_attr(not(test), no_std)]
#[macro_use]
extern crate alloc;

pub const ED_CODE: u64  = 0xed;
pub const SHA2_256_CODE: u64 = 0x12;
pub const CBOR_CODE: u64 = 0x51;

pub mod verification;
pub mod utils;
pub mod cbor;
pub mod bytes;
pub mod error;
pub mod cid;
pub mod wasmsg;
