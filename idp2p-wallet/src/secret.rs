use serde::{Deserialize, Serialize};
use idp2p_common::encode_vec;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct SecretWallet {
    pub(crate) next_index: u32,
    pub(crate) next_secret_index: u32,
    pub(crate) recovery_secret_index: u32,
    #[serde(with = "encode_vec")]
    pub(crate) assertion_secret: Vec<u8>,
    #[serde(with = "encode_vec")]
    pub(crate) authentication_secret: Vec<u8>,
    #[serde(with = "encode_vec")]
    pub(crate) keyagreement_secret: Vec<u8>,
}
