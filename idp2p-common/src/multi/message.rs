use std::io::Read;

use super::{error::Idp2pMultiError, id::Idp2pId};
use unsigned_varint::{encode as varint_encode, io::{read_u64, read_u8}};

#[derive(Debug, Clone, PartialEq)]
pub struct Idp2pMessage {
    pub id: Idp2pId,
    pub body: Vec<u8>,
}

impl Idp2pMessage {
    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, Idp2pMultiError> {
        let mut r = bytes.as_ref();
        let version = read_u64(&mut r)?;
        let codec = read_u64(&mut r)?;
        let cover = read_u64(&mut r)?;
        let _ = read_u64(&mut r)?; // hash code
        let size = read_u8(&mut r)?; // digest size
        let mut digest: Vec<u8> = vec![0; size as usize];
        r.read_exact(&mut digest)?;
        let mut body: Vec<u8> = vec![];
        r.read_to_end(&mut body)?;
        Ok(Self {
            id: Idp2pId::from_fields(version, codec, cover, &digest)?,
            body 
        })
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, Idp2pMultiError> {
        Ok(vec![])
    }
}
