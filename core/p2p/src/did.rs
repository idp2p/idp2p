use alloc::collections::BTreeMap;

use idp2p_common::utils::{from_base32, to_base32};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Bytes(pub Vec<u8>);

impl Serialize for Bytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            return serializer.serialize_str(&to_base32(&self.0)); 
        }else{
            return serializer.serialize_bytes(&self.0);
        }
    }
}

impl<'de> Deserialize<'de> for Bytes {
    fn deserialize<D>(deserializer: D) -> Result<Bytes, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer).unwrap();
            return Ok(Bytes(from_base32(&s)));
        }else{
            let b = Vec::deserialize(deserializer).unwrap();
            return Ok(Bytes(b));
        }
    }
}

impl Bytes {
    pub fn into_boxed_slice(self) -> Box<[u8]> {
        self.0.into_boxed_slice()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Did {
    id: String,
    incepiton: Bytes,
    events: BTreeMap<String, Bytes>,
}