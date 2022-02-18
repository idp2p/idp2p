use idp2p_core::did::Identity;
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
    pub fn new(payload: IdentityMessagePayload) -> IdentityMessage {
        let rnd = idp2p_common::create_random::<32>();
        let id: String = idp2p_common::encode(&rnd);
        IdentityMessage { id, payload }
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

    /*#[test]
    fn handle_get_test() {
        let message = IdentityMessage::new(IdentityMessagePayload::Get);
        let store = MockIdStore{};
        let r = message.handle("did:p2p:1234", &mut map, store);
        assert!(r.is_ok());
    }

    struct MockIdStore {}

    impl IdStore for MockIdStore {
        fn put(&self, key: &str, value: Identity) {}
        fn get(&self, key: &str) -> Option<Identity> {
            Identity::new(&vec![], &vec![])
        }
    }*/
}
