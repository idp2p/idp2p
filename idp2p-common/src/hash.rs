use anyhow::Result;
use cid::{Cid, multihash::{Code, MultihashDigest}};
use multibase::Base;
use serde::Serialize;

#[derive(PartialEq, Debug, Clone)]
pub enum Idp2pHash {
    Sha256,
}

pub enum Idp2pCodec {
    Protobuf = 0x50,
    Json = 0x0200,
}

impl TryFrom<u64> for Idp2pHash {
    type Error = anyhow::Error;

    fn try_from(v: u64) -> Result<Self, Self::Error> {
        match v {
            0x12 => Ok(Idp2pHash::Sha256),
            _ => anyhow::bail!(""),
        }
    }
}

impl Idp2pHash {
    pub fn generate_id(&self, content: &[u8]) -> Vec<u8> {
        match &self {
            Idp2pHash::Sha256 => Code::Sha2_256.digest(&content).digest().to_vec(),
        }
    }
    pub fn generate_cid(&self, content: &[u8], codec: u64) -> Vec<u8> {
        match &self {
            Idp2pHash::Sha256 => {
                let hash = Code::Sha2_256.digest(&content);
                let cid = Cid::new_v1(codec, hash);
                cid.to_bytes()
            }
        }
    }
    pub fn generate_json_cid<T: Sized + Serialize>(&self, content: &T) -> Result<String> {
        let content_bytes = serde_json::to_vec(content).unwrap();
        let cid = self.generate_cid(&content_bytes, Idp2pCodec::Json as u64);
        Ok(multibase::encode(Base::Base64Url, cid))
    }
}


