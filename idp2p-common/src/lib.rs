use cid::{
    multihash::{Code, MultihashDigest},
    Cid,
};
use multibase::*;
use rand::prelude::*;
use serde::Serialize;
use sha2::{Digest, Sha256};

pub const IDP2P_ED25519: &str = "Idp2pEd25519Key";
pub const ED25519: &str = "Ed25519VerificationKey2020";
pub const X25519: &str = "X25519KeyAgreementKey2020";
const JSON_CODEC: u64 = 0x0200;
pub type IdKeySecret = Vec<u8>;
pub type IdKey = Vec<u8>;
pub type IdKeyDigest = Vec<u8>;
pub mod ed_secret;
//pub mod secret;
pub mod encode_vec {
    use multibase::Base;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let (_, data) = multibase::decode(&s).unwrap();
        Ok(data)
    }

    pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: AsRef<[u8]>,
    {
        format!("{}", multibase::encode(Base::Base32Lower, value.as_ref())).serialize(serializer)
    }
}

pub fn encode(value: &[u8]) -> String {
    multibase::encode(Base::Base32Lower, value)
}

pub fn encode_base64url(value: &[u8]) -> String {
    multibase::encode(Base::Base64Url, value)
}

pub fn encode_base64(value: &[u8]) -> String {
    multibase::encode(Base::Base64, value)
}

pub fn decode(s: &str) -> Vec<u8> {
    multibase::decode(s).unwrap().1
}

pub fn decode_<const N: usize>(s: &str) -> anyhow::Result<[u8; N]> {
    let r = multibase::decode(s)?.1;
    let data: [u8; N] = r.try_into().expect("Data size is not equal to given size");
    Ok(data)
}

pub fn hash(bytes: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::default();
    hasher.update(bytes);
    hasher.finalize().to_vec()
}

pub fn generate_cid<T: Sized + Serialize>(t: &T) -> String {
    let content = serde_json::to_string(t).unwrap();
    let hash = Code::Sha2_256.digest(content.as_bytes());
    let cid = Cid::new_v1(JSON_CODEC, hash);
    cid.to_string()
}

pub fn create_random<const N: usize>() -> [u8; N] {
    let mut key_data = [0u8; N];
    let mut key_rng = thread_rng();
    key_rng.fill_bytes(&mut key_data);
    key_data
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn hash_test() {
        let data = json!({
            "name": "John Doe"
        });
        let expected = "botmu6ay364t223hj4akn7amds6rpwquuavx54demvy5e4vkn5uuq";
        let digest = hash(serde_json::to_string(&data).unwrap().as_bytes());
        let result = encode(&digest);
        assert_eq!(result, expected);
    }
    #[test]
    fn cid_test() {
        let data = json!({
            "name": "John Doe"
        });
        let expected = "bagaaieraotmu6ay364t223hj4akn7amds6rpwquuavx54demvy5e4vkn5uuq";
        let cid = generate_cid(&data);
        assert_eq!(cid, expected);
    }
}
