use idp2p_common::store::IdStore;
use idp2p_core::did::Identity;
use serde::{Deserialize, Serialize};

pub enum IdentityEvent {
    IdentityChanged,
    MessageReceived,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct IdentityMessage {
    pub id: String,
    pub payload: IdentityMessagePayload,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(tag = "type")]
pub enum IdentityMessagePayload {
    #[serde(rename = "get")]
    Get,
    #[serde(rename = "post")]
    Post { digest: String, identity: Identity },
    #[serde(rename = "jwm")]
    Jwm { message: String },
}

impl IdentityMessage {
    pub fn new(payload: IdentityMessagePayload) -> IdentityMessage {
        let id: String = idp2p_common::encode(&idp2p_common::create_random::<32>());
        IdentityMessage { id, payload }
    }

    pub fn handle(&self, topic: &str, store: impl IdStore) -> Option<IdentityEvent> {
        match &self.payload {
            IdentityMessagePayload::Get => {
                handle_get(topic, store);
            }
            IdentityMessagePayload::Post { digest, identity } => {
                let changed = handle_post(digest, identity, store);
                if changed {
                    return None;
                }
            }
            IdentityMessagePayload::Jwm { message } => {
                handle_jwm(topic, message, store);
                return None;
            }
        }
        None
    }
}

fn handle_post(digest: &String, identity: &Identity, store: impl IdStore) -> bool {
    true
}
fn handle_get(topic: &str, store: impl IdStore) {}
fn handle_jwm(topic: &str, message: &String, store: impl IdStore) {}

#[cfg(test)]
mod tests {
    use super::*;
}
