use crate::encode_vec;

use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Idp2pProof {
    #[serde(with ="encode_vec")]
    key: Vec<u8>,
    #[serde(with ="encode_vec")]
    value: Vec<u8>
}
