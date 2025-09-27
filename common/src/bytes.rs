use serde::{de::Error as DeError, Deserialize, Deserializer, Serializer};
use serde_with::{DeserializeAs, SerializeAs};
use alloc::{string::String, vec::Vec};

use crate::utils::{decode, encode};

pub struct Bytes;

impl SerializeAs<Vec<u8>> for Bytes {
    fn serialize_as<S>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            return serializer.serialize_str(&encode(bytes)); 
        }else{
            return serializer.serialize_bytes(bytes);
        }
    }
}

impl<'de> DeserializeAs<'de, Vec<u8>> for Bytes {
    fn deserialize_as<D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            return decode(&s).map_err(|e| D::Error::custom(format!("{}", e)));
        } else {
            let b = Vec::deserialize(deserializer)?;
            return Ok(b);
        }
    }
}
