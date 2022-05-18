use cid::{multihash::{Code, MultihashDigest}, Cid};

use crate::Idp2pCodec;

pub trait  Idp2pCid {
    fn new_cid(content: &[u8], codec: Idp2pCodec) -> Cid;
}

impl Idp2pCid for Cid {
    fn new_cid(content: &[u8], codec: Idp2pCodec) -> Cid {
        let mh = Code::Sha2_256.digest(content);
        Cid::new_v1(codec as u64, mh)
    }
}