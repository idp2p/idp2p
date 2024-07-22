
#![no_std]

extern crate alloc;

use alloc::{format, string::String};
pub mod hash;
pub mod cid;
pub mod verifying;

pub fn is_idp2p(id: &str) -> anyhow::Result<bool> {
    let re = regex::Regex::new(r"idp2p:*").map_err(anyhow::Error::msg)?;
    Ok(re.is_match(id))
}

pub fn to_hex_str<T: AsRef<[u8]>>(data: T) -> String{
    format!("0x{}", hex::encode(data))
}