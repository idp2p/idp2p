use anyhow::{bail, Result};
use cid::{multihash::{Code, MultihashDigest}, Cid};

use crate::Idp2pCodec;

pub trait  Idp2pCid {
    fn new_cid(content: &[u8], codec: Idp2pCodec) -> Cid;
    fn ensure(&self, content: &[u8], codec: Idp2pCodec) -> Result<()>;
}

impl Idp2pCid for Cid {
    fn new_cid(content: &[u8], codec: Idp2pCodec) -> Cid {
        new_cid(content, codec)
    }

    fn ensure(&self, content: &[u8], codec: Idp2pCodec) -> Result<()> {
        let ecpected_cid = new_cid(content, codec);
        if ecpected_cid != *self {
            bail!("Invalid cid")
        }
        Ok(())
    }
}

fn new_cid(content: &[u8], codec: Idp2pCodec) -> Cid {
    let mh = Code::Sha2_256.digest(content);
    Cid::new_v1(codec as u64, mh)
}