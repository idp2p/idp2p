use core::did::Identity;
use serde::{Deserialize, Serialize};
use rand::distributions::Alphanumeric;
use rand::prelude::*;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct IdentityMessage {
    pub nonce: String,
    pub command: IdentityCommand,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(tag = "type")]
pub enum IdentityCommand {
    #[serde(rename = "get")]
    Get,
    #[serde(rename = "post")]
    Post(Identity),
}

impl IdentityMessage {
    pub fn new(command: IdentityCommand) -> IdentityMessage {
        let nonce: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();
        IdentityMessage{
            nonce,
            command
        }
    }
}
