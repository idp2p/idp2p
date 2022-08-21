pub mod serde_vec {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use crate::{multi_::base::Idp2pBase, decode_base};
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        decode_base!(s)
    }

    pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: AsRef<[u8]>,
    {
        let s = Idp2pBase::default().encode(value.as_ref());
        format!("{}", s).serialize(serializer)
    }
}
