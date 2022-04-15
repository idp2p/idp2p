use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct JwmHeader {
    pub kid: String,
}

pub mod jwe;
pub mod jwk;
pub mod jws;
pub mod jwm;
pub mod jpm;
