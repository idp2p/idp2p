use cid::Cid;
use regex::Regex;
use sha2::{Digest, Sha256};

pub fn to_hex_str<T: AsRef<[u8]>>(data: T) -> String{
    format!("0x{}", hex::encode(data))
}

pub fn sha256_hash(content: &[u8]) -> anyhow::Result<[u8; 32]> {
    let digest: [u8; 32] = Sha256::digest(content)
        .try_into()
        .map_err(anyhow::Error::msg)?;
    Ok(digest)
}

pub fn parse_id(prefix: &str, id: &str) -> Result<(String, Cid), anyhow::Error> {
    let re_str = format!("/idp2p/{prefix}/(?<major>([0-9]+))/(?<minor>([0-9]+))/(?<cid>(.+))$");
    let re = Regex::new(&re_str).unwrap();
    let Some(caps) = re.captures(id) else {
          anyhow::bail!("");
    };
    let cid = Cid::try_from(caps["cid"].to_string().as_str()).map_err(anyhow::Error::msg)?;
    let version = format!("{}.{}", caps["major"].to_string(), caps["minor"].to_string());
    Ok((version, cid))
}