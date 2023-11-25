use super::{error::Idp2pMultiError, hash::Idp2pMultiHash};
use unsigned_varint::{encode as varint_encode, io::read_u8};

#[derive(Debug, Clone, PartialEq)]
pub enum Idp2pId {
    Id(Vec<u8>),
    IdEvent(Vec<u8>),
    IdSignerKey(Vec<u8>),
    BlockItem(u8, Vec<u8>),
}

impl Idp2pId {
    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, Idp2pMultiError> {
        let mut r = bytes.as_ref();
        
        let typ = read_u8(&mut r)?;
        match typ {
            1 => Ok(Self::Id(r.to_vec())),
            2 => Ok(Self::IdEvent(r.to_vec())),
            3 => Ok(Self::IdSignerKey(r.to_vec())),
            4 => {
                let tx_type = read_u8(&mut r)?;
                Ok(Self::BlockItem(tx_type, r.to_vec()))
            }
            _ => Err(Idp2pMultiError::InvalidId),
        }
    }


    pub fn to_bytes(&self) -> Result<Vec<u8>, Idp2pMultiError> {
        let mut body: Vec<u8> = vec![];

        let typ: u8 = match self {
            Self::Id(mh) => {
                body.extend_from_slice(mh);
                1
            }
            Self::IdEvent(mh) => {
                body.extend_from_slice(mh);
                2
            }
            Self::IdSignerKey(mh) => {
                body.extend_from_slice(mh);
                3
            }
            Self::BlockItem(tx_type, mh) => {
                let mut tx_type_buf = varint_encode::u8_buffer();
                body.extend_from_slice(varint_encode::u8(*tx_type, &mut tx_type_buf));
                body.extend_from_slice(mh);
                4
            }
        };
        let mut typ_buf = varint_encode::u8_buffer();
        Ok([varint_encode::u8(typ, &mut typ_buf), &body].concat())
    }

    /// Ensure the id is produced from the content
    pub fn ensure(&self, content: &[u8]) -> Result<(), Idp2pMultiError> {
        let mh_bytes: &Vec<u8> = match self {
            Self::Id(mh) => mh,
            Self::IdEvent(mh) => mh,
            Self::IdSignerKey(mh) => mh,
            Self::BlockItem(_, mh) => mh,

        };
        let mh = Idp2pMultiHash::from_bytes(&mh_bytes)?;
        mh.ensure(content)?;
        Ok(())
    }
}

/*#[derive(Debug, Clone, PartialEq)]
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

    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, Idp2pMultiError> {
        let mut r = bytes.as_ref();
        let version = read_u64(&mut r)?;
        let codec = read_u64(&mut r)?;
        let cover = read_u64(&mut r)?;
        Self::from_fields(version, codec, cover, r)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut version_buf = varint_encode::u64_buffer();
        let version = varint_encode::u64(self.version, &mut version_buf);
        let mut codec_buf = varint_encode::u64_buffer();
        let codec = varint_encode::u64(self.codec.clone() as u64, &mut codec_buf);
        let mut cover_buf = varint_encode::u64_buffer();
        let cover = varint_encode::u64(self.cover.clone() as u64, &mut cover_buf);
        [version, codec, cover, &self.digest.to_bytes()].concat()
    }

    /// Ensure the id is produced from the content
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
}*/

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn enc_dec_test() -> Result<(), Idp2pMultiError> {
        let id = Idp2pId::Id(vec![]);
        let id2 = Idp2pId::from_bytes(&id.to_bytes()?)?;
        assert_eq!(id, id2);
        Ok(())
    }
}
