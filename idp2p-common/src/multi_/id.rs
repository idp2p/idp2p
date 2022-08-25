use super::{error::Idp2pMultiError, hash::Idp2pHasher};
use cid::multihash::Multihash;
use unsigned_varint::{encode as varint_encode, io::read_u64};

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
    pub version: u64,      // cid version
    pub codec: Idp2pCodec, // codec
    pub cover: u64,        // content version
    pub digest: Multihash, // multihash of content
}

impl Idp2pId {
    pub fn new(cover: u64, content: &[u8]) -> Self {
        Self {
            version: 1,
            codec: Idp2pCodec::Protobuf,
            cover: cover,
            digest: Idp2pHasher::default().digest(content),
        }
    }

    pub fn from_fields(
        version: u64,
        codec: u64,
        cover: u64,
        digest: &[u8],
    ) -> Result<Self, Idp2pMultiError> {
        Ok(Self {
            version: version,
            codec: codec.try_into()?,
            cover: cover,
            digest: Multihash::from_bytes(digest)?,
        })
    }

    pub fn decode<T: AsRef<[u8]>>(bytes: T) -> Result<Self, Idp2pMultiError> {
        let mut r = bytes.as_ref();
        let version = read_u64(&mut r)?;
        let codec = read_u64(&mut r)?;
        let cover = read_u64(&mut r)?;
        Self::from_fields(version, codec, cover, r)
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut version_buf = varint_encode::u64_buffer();
        let version = varint_encode::u64(self.version, &mut version_buf);
        let mut codec_buf = varint_encode::u64_buffer();
        let codec = varint_encode::u64(self.codec.clone() as u64, &mut codec_buf);
        let mut cover_buf = varint_encode::u64_buffer();
        let cover = varint_encode::u64(self.cover.clone() as u64, &mut cover_buf);
        [version, codec, cover, &self.digest.to_bytes()].concat()
    }

    pub fn ensure(&self, content: &[u8]) -> Result<(), Idp2pMultiError> {
        let hasher: Idp2pHasher = self.digest.code().try_into()?;
        let expected_id = Self {
            version: self.version,
            codec: self.codec.clone(),
            cover: self.cover,
            digest: hasher.digest(content),
        };
        if expected_id != *self {
            return Err(Idp2pMultiError::InvalidCid);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn enc_dec_test() -> Result<(), Idp2pMultiError> {
        let id = Idp2pId::new(0, &vec![0u8; 50]);
        let id2 = Idp2pId::decode(&id.encode())?;
        assert_eq!(id, id2);
        Ok(())
    }
}