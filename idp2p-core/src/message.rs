use crate::did::Identity;
use serde::{Deserialize, Serialize};

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
    pub fn new(payload: IdentityMessagePayload) -> Self {
        let rnd = idp2p_common::create_random::<32>();
        let id: String = idp2p_common::encode(&rnd);
        IdentityMessage { id, payload }
    }

    pub fn new_post(did: Identity) -> Self {
        let payload = IdentityMessagePayload::Post {
            digest: did.get_digest(),
            identity: did,
        };
        Self::new(payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_test() {
        let message = IdentityMessage::new(IdentityMessagePayload::Get);
        assert_eq!(idp2p_common::decode(&message.id).len(), 32);
    }
}
