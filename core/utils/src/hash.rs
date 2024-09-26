use sha2::{Digest, Sha256};

pub fn sha256_hash(content: &[u8]) -> anyhow::Result<[u8; 32]> {
    let digest: [u8; 32] = Sha256::digest(content)
        .try_into()
        .map_err(anyhow::Error::msg)?;
    Ok(digest)
}