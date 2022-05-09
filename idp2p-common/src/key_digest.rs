use anyhow::{Result, bail};
use multihash::Multihash;
use serde::{Serialize, Deserialize};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum Idp2pKeyDigest {
    Idp2pEd25519 {
        digest: Vec<u8>
    }
}

impl Idp2pKeyDigest{
    pub fn try_from(code: i32, digest: &[u8]) -> Result<Self>{
         match  code {
             0xed => {
                 Ok(Idp2pKeyDigest::Idp2pEd25519 { digest: digest.to_vec() })
             }
             _ => bail!("")
         }
    }
}


/*impl FromStr for Idp2pKeyDigest {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_, key_data) = multibase::decode(&s).unwrap();
        Ok(key_data.try_into().unwrap())
    }
}

impl TryFrom<Vec<u8>> for Idp2pKeyDigest {
    type Error = anyhow::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if value.len() < 34 {
            bail!("Length should be 66 bytes or more")
        }
        Ok(Self {
            key_type: value[0],
            digest: value[1..].to_vec().try_into()?,
        })
    }
}

impl From<Idp2pKeyDigest> for Vec<u8> {
    fn from(value: Idp2pKeyDigest) -> Self {
        let mut encoded: Vec<u8> = vec![value.key_type];
        let digest: Vec<u8> = value.digest.into();
        encoded.extend_from_slice(&digest);
        encoded
    }
}

impl Serialize for Idp2pKeyDigest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let encoded: Vec<u8> = self.to_owned().into();
        let s = multibase::encode(multibase::Base::Base64Url, encoded);
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for Idp2pKeyDigest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        Ok(Self::from_str(s).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn try_from_test() {
        let mut bytes = [0u8; 34];
        bytes[2..].copy_from_slice(&[1u8; 32]);
        let digest: Idp2pKeyDigest = bytes.to_vec().try_into().unwrap();
        assert_eq!(digest.key_type, 0);
    }

    #[test]
    fn encode_test() {
        let proof = Idp2pKeyDigest {
            key_type: 0,
            digest: Idp2pDigest::Sha256 { digest: [0u8; 32] },
        };
        let vec: Vec<u8> = proof.into();
        assert_eq!(vec.len(), 34);
    }
}*/
