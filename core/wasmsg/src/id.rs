use serde::{Deserialize, Serialize};
use sha2::{digest::Digest, Sha256};

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize, Clone)]
pub enum DigestId {
    Sha256([u8; 32]),
}

impl DigestId {
    pub fn new_sha256(content: &[u8]) -> Result<Self, &'static str> {
        let digest: [u8; 32] = Sha256::digest(content)
            .try_into()
            .map_err(|_| "RUNTIME_ERROR")?;
        Ok(DigestId::Sha256(digest))
    }

    pub fn ensure(&self, content: &[u8]) -> Result<(), &'static str> {
        match self {
            DigestId::Sha256(digest) => {
                let expected: [u8; 32] = Sha256::digest(content)
                    .try_into()
                    .map_err(|_| "RUNTIME_ERROR")?;
                if &expected == digest {
                    return Err("DIGEST_ERROR");
                }
            }
        }
        Ok(())
    }
}

/*mod serde_bytes_array {
    use core::convert::TryInto;
    use alloc::format;
    use serde::de::Error;
    use serde::{Deserializer, Serializer};

    /// This just specializes [`serde_bytes::serialize`] to `<T = [u8]>`.
    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serde_bytes::serialize(bytes, serializer)
    }

    /// This takes the result of [`serde_bytes::deserialize`] from `[u8]` to `[u8; N]`.
    pub fn deserialize<'de, D, const N: usize>(deserializer: D) -> Result<[u8; N], D::Error>
    where
        D: Deserializer<'de>,
    {
        let slice: &[u8] = serde_bytes::deserialize(deserializer)?;
        let array: [u8; N] = slice.try_into().map_err(|_| {
            let expected = format!("[u8; {}]", N);
            D::Error::invalid_length(slice.len(), &expected.as_str())
        })?;
        Ok(array)
    }
}*/