use std::io::Read;

use super::{error::Idp2pMultiError, hash::Idp2pHash};
use cid::multihash::Multihash;
use unsigned_varint::{encode as varint_encode, io::read_u64, io::read_u8};

#[derive(Debug, Clone, PartialEq)]
pub enum Idp2pCodec {
    Protobuf = 0x50,
    Json = 0x0200,
}

impl TryFrom<u64> for Idp2pCodec {
    type Error = Idp2pMultiError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0x50 => Ok(Self::Protobuf),
            0x0200 => Ok(Self::Json),
            _ => Err(Idp2pMultiError::InvalidKeyCode),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Idp2pId {
    pub version: u64,      // idp2p version
    pub codec: Idp2pCodec, // codec
    pub mh: Multihash,     // multihash of content
}

impl Idp2pId {
    pub fn new(codec: Idp2pCodec, hasher: Idp2pHash, content: &[u8]) -> Self {
        Self {
            version: 0,
            codec,
            mh: hasher.digest(content),
        }
    }

    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, Idp2pMultiError> {
        let mut r = bytes.as_ref();
        let version = read_u64(&mut r)?; // idp2p version
        let codec = read_u64(&mut r)?; // content encoding
        let code = read_u64(&mut r)?; // hash code
        let mut code_buf = varint_encode::u64_buffer();
        let code_bytes = varint_encode::u64(code, &mut code_buf);
        let size = read_u8(&mut r)?; // digest size
        let mut digest: Vec<u8> = vec![0; size as usize];
        r.read_exact(&mut digest)?;
        Ok(Self {
            version: version,
            codec: codec.try_into()?,
            mh: Multihash::from_bytes(&[code_bytes, &[size], &digest].concat())?,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut version_buf = varint_encode::u64_buffer();
        let version = varint_encode::u64(self.version, &mut version_buf);
        let mut codec_buf = varint_encode::u64_buffer();
        let codec = varint_encode::u64(self.codec.clone() as u64, &mut codec_buf);
        [version, codec, &self.mh.to_bytes()].concat()
    }

    pub fn ensure(&self, content: &[u8]) -> Result<(), Idp2pMultiError> {
        let hasher = self.mh.code().try_into()?;
        let expected_id = Self::new(self.codec.clone(), hasher, content);
        if expected_id != *self {
            return Err(Idp2pMultiError::InvalidCid);
        }
        Ok(())
    }
}

/*pub trait Idp2pCid {
    fn from_bytes(bytes: &[u8]) -> Result<Cid, Idp2pMultiError>;
    fn new_cid(codec: Idp2pCodec, content: &[u8]) -> Cid;
    fn ensure(&self, content: &[u8]) -> Result<(), Idp2pMultiError>;
}

impl Idp2pCid for Cid {
    fn new_cid(codec: Idp2pCodec, content: &[u8]) -> Cid {
        let mh = Idp2pHash::default().digest(content);
        Cid::new_v1(codec as u64, mh)
    }

    fn ensure(&self, content: &[u8]) -> Result<(), Idp2pMultiError> {
        let hash = Idp2pHash::try_from(self.hash().code())?;
        let mh = hash.digest(content);
        let expected_cid = Cid::new_v1(self.codec(), mh);
        if expected_cid != *self {
            return Err(Idp2pMultiError::InvalidCid);
        }
        Ok(())
    }

    fn from_bytes(bytes: &[u8]) -> Result<Cid, Idp2pMultiError> {
        Ok(bytes.try_into()?)
    }
}*/
