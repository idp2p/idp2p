use cid::Cid;
use multibase::Base;
use multihash::{Code, MultihashDigest};
use serde::Serialize;
use anyhow::Result;

#[derive(PartialEq, Debug, Clone)]
pub enum Idp2pHash {
    Sha256,
}

pub enum Idp2pCodec {
    Protobuf = 0x50,
    Json = 0x0200,
}

impl TryFrom<i32> for Idp2pHash {
    type Error = anyhow::Error;

    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
           0x12 => Ok(Idp2pHash::Sha256),
           _ => anyhow::bail!("")
        }
    }
}

pub fn generate_id(content: &[u8], hash_alg: Idp2pHash) -> Vec<u8> {
    match &hash_alg {
        Idp2pHash::Sha256 => Code::Sha2_256.digest(&content).digest().to_vec(),
    }
}

pub fn generate_cid(content: &[u8], codec: Idp2pCodec, hash_type: Idp2pHash) -> Vec<u8> {
    match &hash_type {
        Idp2pHash::Sha256 => {
            let hash = Code::Sha2_256.digest(&content);
            let cid = Cid::new_v1(codec as u64, hash);
            cid.to_bytes()
        }
    }
}

pub fn generate_json_cid<T: Sized + Serialize>(content: &T) -> Result<String> {
    let content_bytes = serde_json::to_vec(content).unwrap();
    let cid = generate_cid(&content_bytes, Idp2pCodec::Json, Idp2pHash::Sha256);
    Ok(multibase::encode(Base::Base64Url, cid))
}
