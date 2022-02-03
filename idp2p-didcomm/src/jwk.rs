use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Jwk {
    pub kty: String,
    pub crv: String,
    pub x: String,
}

impl Jwk {
    pub fn from_bytes(bytes: &[u8]) -> Jwk {
        Jwk {
            kty: "OKP".to_owned(),
            crv: "X25519".to_owned(),
            x: idp2p_common::encode_base64url(bytes),
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        idp2p_common::decode(&self.x)
    }
}
