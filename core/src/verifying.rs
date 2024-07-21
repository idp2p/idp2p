use cid::Cid;
use ed25519_dalek::{Signature, VerifyingKey};

use crate::cid::CidExt;

pub const ED_CODE: u64 = 0xed;
const ED25519_PUBKEY_SIZE: usize = 32;
const ED25519_SIG_SIZE: usize = 64;

#[derive(Debug)]
pub struct Ed25519PublicKey(pub [u8; ED25519_PUBKEY_SIZE]);

impl Ed25519PublicKey {
    pub fn to_id(&self) -> anyhow::Result<Cid> {
        Cid::create(ED_CODE, &self.0)
    }

    pub fn from_bytes(public: &[u8]) -> anyhow::Result<Self> {
        let pub_bytes = public.try_into().map_err(anyhow::Error::msg)?;
        Ok(Ed25519PublicKey(pub_bytes))
    }

    pub fn verify(&self, content: &[u8], sig: [u8; ED25519_SIG_SIZE]) -> anyhow::Result<bool> {
        let pk = VerifyingKey::from_bytes(&self.0).map_err(anyhow::Error::msg)?;
        let signature = Signature::from(sig);
        pk.verify_strict(content, &signature)
            .map_err(anyhow::Error::msg)?;
        return Ok(true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn to_bytes_test() {}
}