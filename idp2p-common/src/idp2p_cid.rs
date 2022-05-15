use crate::{Idp2pCodec, hasher::Idp2pHasher};
use anyhow::{bail, Result};
use cid::{
    multihash::{Code, MultihashDigest},
    Cid,
};

pub struct Idp2pCid(Vec<u8>);

impl Idp2pCid {
    pub fn new(content: &[u8], codec: Idp2pCodec) -> Self {
        let mh = Code::Sha2_256.digest(content);
        let id = Cid::new_v1(codec as u64, mh);
        Self(id.to_bytes())
    }

    pub fn ensure(&self, content: &[u8], codec: Idp2pCodec) -> Result<()> {
        let cid: Cid = self.0.clone().try_into()?;
        if cid.codec() != codec as u64 {
            bail!("Invalid codec")
        }
        if !cid.hash().is_hash_of(content)? {
            bail!("Invalid cid")
        }
        Ok(())
    }
}
