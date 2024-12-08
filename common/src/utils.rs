use sha2::{Digest, Sha256};

pub fn is_idp2p(id: &str) -> anyhow::Result<bool> {
    let re = regex::Regex::new(r"idp2p:*").map_err(anyhow::Error::msg)?;
    Ok(re.is_match(id))
}

pub fn to_hex_str<T: AsRef<[u8]>>(data: T) -> String{
    format!("0x{}", hex::encode(data))
}

pub fn sha256_hash(content: &[u8]) -> anyhow::Result<[u8; 32]> {
    let digest: [u8; 32] = Sha256::digest(content)
        .try_into()
        .map_err(anyhow::Error::msg)?;
    Ok(digest)
}
