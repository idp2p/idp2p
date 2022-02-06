use anyhow::Result;
use idp2p_common::store::IdStore;
use idp2p_core::did::Identity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum IdentityMessageResult {
    Skipped,
    Created { id: String },
    Updated { id: String },
    Requested { message: IdentityMessage },
    ReceivedJwm { id: String, jwm: String },
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
        let rnd = idp2p_common::create_random::<32>();
        let id: String = idp2p_common::encode(&rnd);
        IdentityMessage { id, payload }
    }

    pub fn handle(
        &self,
        topic: &str,
        identities: &mut HashMap<String, String>,
        store: impl IdStore,
    ) -> Result<IdentityMessageResult> {
        match &self.payload {
            IdentityMessagePayload::Get => {
                let identity: Identity = store.get("identities", &topic).unwrap();
                let payload = IdentityMessagePayload::Post {
                    digest: identity.get_digest(),
                    identity: identity.clone(),
                };
                let mes = IdentityMessage::new(payload);
                Ok(IdentityMessageResult::Requested { message: mes })
            }
            IdentityMessagePayload::Post { digest, identity } => {
                let current = identities.get(&identity.id);
                match current {
                    None => {
                        identity.verify()?;
                        identities.insert(identity.id.clone(), identity.get_digest());
                        store.put("identities", &identity.id, identity.clone());
                        Ok(IdentityMessageResult::Created {
                            id: identity.id.clone(),
                        })
                    }
                    Some(current_digest) => {
                        if digest == current_digest {
                            return Ok(IdentityMessageResult::Skipped);
                        }
                        let current_did: Identity = store.get("identities", &identity.id).unwrap();
                        current_did.is_next(identity.clone())?;
                        identities.insert(identity.id.clone(), identity.get_digest());
                        store.put("identities", &identity.id, identity.clone());
                        Ok(IdentityMessageResult::Updated {
                            id: identity.id.clone(),
                        })
                    }
                }
            }
            IdentityMessagePayload::Jwm { message } => {
                let result = IdentityMessageResult::ReceivedJwm {
                    id: topic.to_owned(),
                    jwm: message.to_owned(),
                };
                Ok(result)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::DeserializeOwned;
    #[test]
    fn new_test() {
        let message = IdentityMessage::new(IdentityMessagePayload::Get);
        assert_eq!(idp2p_common::decode(&message.id).len(), 32);
    }

    #[test]
    fn handle_get_test() {
        let message = IdentityMessage::new(IdentityMessagePayload::Get);
        let store = MockIdStore::new();
        let mut map: HashMap<String, String> = HashMap::new();
        let r = message.handle("did:p2p:1234", &mut map, store);
        assert!(r.is_ok());
    }

    struct MockIdStore {
        data: HashMap<String, String>,
    }
    impl MockIdStore {
        fn new() -> MockIdStore {
            let mut s = MockIdStore {
                data: HashMap::new(),
            };
            let id = Identity::new(&vec![], &vec![]);
            let v = serde_json::to_string(&id).unwrap();
            let k = format!("{}/{}", "identities", "did:p2p:1234");
            s.data.insert(k, v);
            s
        }
    }
    impl IdStore for MockIdStore {
        fn put<T: Serialize>(&self, entity: &str, key: &str, value: T) {}
        fn get<T: DeserializeOwned>(&self, entity: &str, key: &str) -> Option<T> {
            let k = format!("{}/{}", entity, key);
            let s = self.data.get(&k);
            if s.is_none() {
                return None;
            }
            Some(serde_json::from_str::<T>(&s.unwrap()).unwrap())
        }
    }
}
