use super::{error::Idp2pMultiError, hash::Idp2pMultiHash};
use unsigned_varint::{encode as varint_encode, io::read_u8};

/// First bytes of bytes describes the version of id
/// 0 -> Following bytes is a multihash of payload 

#[derive(Debug, Clone, PartialEq)]
pub struct Id(Idp2pMultiHash);

/// [context-len][context][inception-len][inception]
#[derive(Debug, Clone, PartialEq)]
pub struct IdPayload {
   /// Content identifier(cid) of context document
   context: Vec<u8>, 
   /// Raw inception bytes
   inception: Vec<u8>
}

impl Id {
    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, Idp2pMultiError> {
        let mut r = bytes.as_ref();
        let version = read_u8(&mut r)?;
        assert_eq!(version, 0);
        Ok(Self(Idp2pMultiHash::from_bytes(r)?))
    }
}