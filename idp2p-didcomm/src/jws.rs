use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Jws {
    pub kid: String, // sender kid
    pub alg: String, // EdDSA
    pub payload: String,
    pub signature: String,
}
