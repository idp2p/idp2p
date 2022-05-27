pub mod message;
pub mod random;
pub mod base;
pub mod keys;
pub mod secret;
pub mod identity;
pub mod idp2p_proto {
    include!(concat!(env!("OUT_DIR"), "/idp2p.pb.rs"));
}
pub mod serde_vec {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use crate::base::{decode, encode};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let result = decode(&s);
        match result {
            Ok(data) => Ok(data),
            Err(err) => serde::de::Error::custom(err.to_string()),
        }
    }

    pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: AsRef<[u8]>,
    {
        format!("{}", encode(value.as_ref())).serialize(serializer)
    }
}
pub use prost;
pub use thiserror;
pub use libp2p;