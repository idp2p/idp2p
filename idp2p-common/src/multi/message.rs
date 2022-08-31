use std::io::Read;

use super::{error::Idp2pMultiError, id::Idp2pId};
use unsigned_varint::{
    io::{read_u64, read_u8},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Idp2pMessage {
    pub id: Idp2pId,
    pub body: Vec<u8>,
}

impl Idp2pMessage {
    pub fn new(body: &[u8]) -> Self {
        Self {
            id: Idp2pId::new(0, body),
            body: body.to_vec(),
        }
    }
    pub fn from_multi_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, Idp2pMultiError> {
        let mut r = bytes.as_ref();
        let version = read_u64(&mut r)?;
        let codec = read_u64(&mut r)?;
        let cover = read_u64(&mut r)?;
        read_u64(&mut r)?; // hash code
        let size = read_u8(&mut r)?; // digest size
        let mut digest: Vec<u8> = vec![0; size as usize];
        r.read_exact(&mut digest)?;
        let mut body: Vec<u8> = vec![];
        r.read_to_end(&mut body)?;
        Ok(Self {
            id: Idp2pId::from_fields(version, codec, cover, &digest)?,
            body,
        })
    }

    pub fn to_multi_bytes(&self) -> Result<Vec<u8>, Idp2pMultiError> {
        Ok([&self.id.to_bytes()[..], &self.body[..]].concat())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn enc_dec_test() -> Result<(), Idp2pMultiError> {
        let msg = Idp2pMessage::new(&vec![0u8;20]);
        let msg_bytes = msg.to_multi_bytes()?;
        let dec_msg = Idp2pMessage::from_multi_bytes(msg_bytes)?;
        assert_eq!(msg, dec_msg);
        Ok(())
    }
}