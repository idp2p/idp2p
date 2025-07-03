#![no_std]
extern crate alloc;

pub const ED_CODE: u64 = 0xed;
pub const SHA2_256_CODE: u64 = 0x12;
pub const CBOR_CODE: u64 = 0x51;

pub mod ed25519;
pub mod utils;
pub mod cbor;
pub mod bytes;
pub mod error;
pub mod cid;
