use cid::Cid;

use super::hash::Idp2pHash;

pub enum Idp2pCodec {
    Protobuf = 0x50,
    Json = 0x0200,
}

pub struct Idp2pEventId(Vec<u8>);

pub trait Idp2pCid {
    fn new_cid(codec: Idp2pCodec, content: &[u8]) -> Cid;
}

impl Idp2pCid for Cid {
    fn new_cid(codec: Idp2pCodec, content: &[u8]) -> Cid {
        let mh = Idp2pHash::default().digest(content);
        Cid::new_v1(codec as u64, mh)
    }
}
