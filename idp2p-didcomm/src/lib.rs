use idp2p_core::did::Identity;
use crate::jwm::Jwm;
use serde::{Deserialize, Serialize};
use idp2p_common::serde_json;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct JwmHeader {
    pub kid: String,
}

pub mod jwe;
pub mod jwk;
pub mod jws;
pub mod jwm;
pub mod jpm;
pub mod vcs;
