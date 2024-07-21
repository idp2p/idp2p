
#![no_std]

pub mod hash;
pub mod cid;
pub mod verifying;

pub fn is_idp2p(id: &str) -> anyhow::Result<bool> {
    let re = regex::Regex::new(r"idp2p:*").map_err(anyhow::Error::msg)?;
    Ok(re.is_match(id))
}