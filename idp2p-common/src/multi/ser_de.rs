use serde::{de::Error as SerdeError, Deserialize, Serialize};

impl Serialize for Idp2pKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = Idp2pBase::default().encode(self.to_bytes());
        format!("{}", s).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Idp2pKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = decode_base!(s)?;
        Ok(Self::from_bytes(bytes).map_err(SerdeError::custom)?)
    }
}

impl Serialize for Idp2pAgreementKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = Idp2pBase::default().encode(self.to_bytes());
        format!("{}", s).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Idp2pAgreementKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = decode_base!(s)?;
        Ok(Self::from_bytes(bytes).map_err(SerdeError::custom)?)
    }
}


impl Serialize for Idp2pKeyDigest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = Idp2pBase::default().encode(self.to_bytes());
        format!("{}", s).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Idp2pKeyDigest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = decode_base!(s)?;
        Ok(Self::from_bytes(bytes).map_err(SerdeError::custom)?)
    }
}