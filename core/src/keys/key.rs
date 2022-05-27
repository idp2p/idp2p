
use ed25519_dalek::PublicKey;
use serde::{Serialize, Deserialize};

#[derive(PartialEq, Clone, Debug)]
pub enum Idp2pKey {
    Ed25519 { public: PublicKey },
}

impl Idp2pKey{
    fn to_bytes(&self) -> Vec<u8>{
        
    }
}

impl Serialize for Idp2pKey{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        todo!()
    }
}

impl<'de> Deserialize<'de> for Idp2pKey{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        todo!()
    }
}