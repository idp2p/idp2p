use anyhow::Result;
use multibase::*;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn decode_sized<const N: usize>(s: &str) -> anyhow::Result<[u8; N]> {
    let r = multibase::decode(s)?.1;
    let data: [u8; N] = r.try_into().expect("Data size is not equal to given size");
    Ok(data)
}

pub fn encode<T: Serialize>(value: T) -> Result<String> {
    let s = serde_json::to_string(&value)?;
    let mb64 = multibase::encode(Base::Base64Url, s.as_bytes());
    Ok(mb64[1..].to_owned())
}

pub fn decode<T: DeserializeOwned>(value: &str) -> Result<T> {
    let m64 = format!("u{}", value);
    let s = multibase::decode(&m64)?.1;
    let bytes = std::str::from_utf8(&s)?;
    let t: T = serde_json::from_str(&bytes)?;
    Ok(t)
}

pub fn decode_str(value: &str) -> Result<Vec<u8>> {
    let m64 = format!("u{}", value);
    let vec = multibase::decode(&m64)?.1;
    Ok(vec)
}

pub fn encode_bytes(value: &[u8]) -> Result<String> {
    let mb64 = multibase::encode(Base::Base64Url, value);
    Ok(mb64[1..].to_owned())
}

pub fn encode_str(value: &str) -> Result<String> {
    let mb64 = multibase::encode(Base::Base64Url, value.as_bytes());
    Ok(mb64[1..].to_owned())
}

pub mod encode_vec {
    use multibase::Base;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let (_, data) = multibase::decode(&s).map_err(|_| serde::de::Error::custom(""))?;
        Ok(data)
    }

    pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: AsRef<[u8]>,
    {
        format!("{}", multibase::encode(Base::Base64Url, value.as_ref())).serialize(serializer)
    }
}


