use anyhow::Result;
use multibase::*;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn encode<T: Serialize>(value: T) -> Result<String> {
    let s = serde_json::to_string(&value)?;
    let mb64 = multibase::encode(Base::Base64Url, s.as_bytes());
    Ok(mb64[1..].to_owned())
}

pub fn decode<T: DeserializeOwned>(value: &str) -> Result<T> {
    let m64 = format!("u{}", value);
    let s = crate::decode(&m64);
    let bytes = std::str::from_utf8(&s)?;
    let t: T = serde_json::from_str(&bytes)?;
    Ok(t)
}

pub fn encode_bytes(value: &[u8]) -> Result<String> {
    let mb64 = multibase::encode(Base::Base64Url, value);
    Ok(mb64[1..].to_owned())
}

pub fn encode_str(value: &str) -> Result<String> {
    let mb64 = multibase::encode(Base::Base64Url, value.as_bytes());
    Ok(mb64[1..].to_owned())
}