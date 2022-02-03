use idp2p_common::encode_vec;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Account {
    pub name: String,
    pub id: String,
    #[serde(with = "encode_vec")]
    pub next_secret: Vec<u8>,
    #[serde(with = "encode_vec")]
    pub authentication_secret: Vec<u8>,
    #[serde(with = "encode_vec")]
    pub keyagreement_secret: Vec<u8>,
}
