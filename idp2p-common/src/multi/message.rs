use std::io::Read;

use super::{error::Idp2pMultiError, id::Idp2pCodec};
use unsigned_varint::{encode as varint_encode, io::read_u64};

#[derive(Debug, Clone, PartialEq)]
pub struct Idp2pMessage {
    pub version: u64,
    pub codec: Idp2pCodec,
    pub body: Vec<u8>,
}

impl Idp2pMessage {
    pub fn new_proto_message(body: &[u8]) -> Self {
        Self {
            version: 0,
            codec: Idp2pCodec::Protobuf,
            body: body.to_vec(),
        }
    }

    pub fn from_multi_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, Idp2pMultiError> {
        let mut r = bytes.as_ref();
        let version = read_u64(&mut r)?;
        let codec = read_u64(&mut r)?;
        let size = read_u64(&mut r)?; // digest size
        let mut body: Vec<u8> = vec![0; size as usize];
        r.read_exact(&mut body)?;
        Ok(Self {
            version,
            codec: codec.try_into()?,
            body,
        })
    }

    pub fn to_multi_bytes(&self) -> Result<Vec<u8>, Idp2pMultiError> {
        let mut version_buf = varint_encode::u64_buffer();
        let version = varint_encode::u64(self.version, &mut version_buf);
        let mut codec_buf = varint_encode::u64_buffer();
        let codec = varint_encode::u64(self.codec.clone() as u64, &mut codec_buf);
        let mut size_buf = varint_encode::u64_buffer();
        let size = varint_encode::u64(self.body.len() as u64, &mut size_buf);
        Ok([version, codec, size, &self.body[..]].concat())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn enc_dec_test() -> Result<(), Idp2pMultiError> {
        let msg = Idp2pMessage::new_proto_message(&vec![0u8; 20]);
        let msg_bytes = msg.to_multi_bytes()?;
        let dec_msg = Idp2pMessage::from_multi_bytes(msg_bytes)?;
        assert_eq!(msg, dec_msg);
        Ok(())
    }
}
