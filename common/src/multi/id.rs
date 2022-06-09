use cid::Cid;

use super::{hash::Idp2pHash, error::Idp2pMultiError};

pub enum Idp2pCodec {
    Protobuf = 0x50,
    Json = 0x0200,
}

impl TryFrom<u64> for Idp2pCodec{
    type Error = Idp2pMultiError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value{
            0x50 => Ok(Self::Protobuf),
            0x0200 => Ok(Self::Json),
            _ => Err(Idp2pMultiError::InvalidKeyCode)
        }
    }
}

pub trait Idp2pCid {
    fn from_bytes(bytes: &[u8]) -> Result<Cid, Idp2pMultiError>;
    fn new_cid(codec: Idp2pCodec, content: &[u8]) -> Cid;
    fn ensure(&self, content: &[u8]) -> Result<(), Idp2pMultiError>;
}

impl Idp2pCid for Cid {
    fn new_cid(codec: Idp2pCodec, content: &[u8]) -> Cid {
        let mh = Idp2pHash::default().digest(content);
        Cid::new_v1(codec as u64, mh)
    }

    fn ensure(&self, content: &[u8]) -> Result<(), Idp2pMultiError>{
        let hash = Idp2pHash::try_from(self.hash().code())?;
        let mh = hash.digest(content);
        let expected_cid =  Cid::new_v1(self.codec(), mh);
        if expected_cid != *self {
            return Err(Idp2pMultiError::InvalidCid);
        }
        Ok(())
    }

    fn from_bytes(bytes: &[u8]) -> Result<Cid, Idp2pMultiError> {
        Ok(bytes.try_into()?)
    }
}
