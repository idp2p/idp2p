use crate::did::Identity;
use crate::message::IdentityMessage;
use crate::IdentityEvent;
use idp2p_common::anyhow::Result;
use idp2p_common::chrono::Utc;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

pub struct IdStore {
    pub state: Mutex<IdState>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct IdState {
    pub entries: HashMap<String, IdEntry>,
    pub publish_queue: BTreeMap<(Instant, String), IdentityMessage>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct IdEntry {
    pub is_hosted: bool,
    pub digest: String,
    pub did: Identity,
    pub last_updated: i64,
    pub last_published: i64,
}

impl IdStore {
    pub fn new(entries: HashMap<String, IdEntry>) -> Self {
        IdStore {
            state: Mutex::new(IdState::new(entries)),
        }
    }

    pub fn get_did(&self, id: &str) -> Identity {
        let state = self.state.lock().unwrap();
        let entry = state.entries.get(id).map(|entry| entry.clone());
        entry.unwrap().did
    }

    pub fn push_did(&self, did: Identity) {
        let mut state = self.state.lock().unwrap();
        let id = did.id.clone();
        let entry = IdEntry::new(did);
        state.entries.insert(id, entry);
    }

    pub fn handle_get(&self, id: &str) -> Option<IdentityMessage> {
        let mut state = self.state.lock().unwrap();
        let entry = state.entries.get(id).map(|entry| entry.clone());
        if let Some(entry) = entry {
            let mes = IdentityMessage::new_post(entry.did.clone());
            if entry.is_hosted {
                return Some(mes);
            } else {
                let when = Instant::now();
                state.publish_queue.insert((when, id.to_owned()), mes);
            }
        }
        None
    }

    pub fn handle_post(&self, digest: &str, identity: &Identity) -> Result<IdentityEvent> {
        let mut state = self.state.lock().unwrap();
        let current = state.entries.get(&identity.id).map(|entry| entry.clone());
        match current {
            None => {
                identity.verify()?;
                let entry = IdEntry::from(identity.clone());
                state.entries.insert(identity.id.clone(), entry);
                return Ok(IdentityEvent::Created {
                    id: identity.id.clone(),
                });
            }
            Some(entry) => {
                if digest != entry.digest {
                    entry.did.is_next(identity.clone())?;
                    let new_entry = IdEntry {
                        did: identity.clone(),
                        ..entry
                    };
                    state.entries.insert(identity.id.clone(), new_entry);
                    return Ok(IdentityEvent::Updated {
                        id: identity.id.clone(),
                    });
                } else {
                    return Ok(IdentityEvent::Skipped {
                        id: identity.id.clone(),
                    });
                }
            }
        }
    }
}

impl IdState {
    pub fn new(entries: HashMap<String, IdEntry>) -> Self {
        IdState {
            entries: entries,
            publish_queue: BTreeMap::new(),
        }
    }
}

impl IdEntry {
    pub fn new(did: Identity) -> Self {
        IdEntry {
            digest: did.get_digest(),
            last_updated: Utc::now().timestamp(),
            last_published: Utc::now().timestamp(),
            is_hosted: true,
            did: did,
        }
    }

    pub fn from(did: Identity) -> Self {
        IdEntry {
            digest: did.get_digest(),
            last_published: Utc::now().timestamp(),
            last_updated: Utc::now().timestamp(),
            is_hosted: false,
            did: did,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use idp2p_common::ed_secret::EdSecret;

    #[test]
    fn handle_get_test() {
        let store = create_store();
        let r = store.handle_get("did:p2p:bagaaierab2xsn6stgdcwfv3wvot7lboh2aewqqjfy56gzh7sibt7vxxtup4q");
        assert!(r.is_some());
    }

    fn create_store() -> IdStore {
        let secret_str = "beilmx4d76udjmug5ykpy657qa3pfsqbcu7fbbtuk3mgrdrxssseq";
        let secret = EdSecret::from_str(secret_str).unwrap();
        let ed_key_digest = secret.to_publickey_digest().unwrap();
        let did = Identity::new(&ed_key_digest, &ed_key_digest);
        let store = IdStore::new(HashMap::new());
        store.push_did(did);   
        store
    }
}
