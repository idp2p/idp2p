use crate::did::Identity;
use crate::message::IdentityMessage;
use crate::IdentityEvent;
use idp2p_common::anyhow::Result;
use idp2p_common::chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::mpsc::Sender;
use tokio::time::{sleep, Duration};

pub struct IdStore {
    pub state: Mutex<IdState>,
    pub event_sender: Sender<IdentityEvent>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct IdState {
    pub entries: HashMap<String, IdEntry>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct IdEntry {
    pub digest: String,
    pub did: Identity,
    pub last_updated: i64,
    pub last_published: i64,
    pub is_hosted: bool,
}

impl IdStore {
    pub fn new(ids: HashMap<String, IdEntry>, es: Sender<IdentityEvent>) -> Self {
        IdStore {
            state: Mutex::new(IdState::new(ids)),
            event_sender: es,
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

    pub fn get_message(&self, id: &str) -> Option<IdentityMessage> {
        let state = self.state.lock().unwrap();
        let entry = state.entries.get(id).map(|entry| entry.clone());
        if let Some(entry) = entry {
            return Some(IdentityMessage::new_post(entry.did));
        }
        None
    }

    pub async fn handle_get(&self, id: &str) {
        let state = self.state.lock().unwrap();
        let entry = state.entries.get(id).map(|entry| entry.clone());
        if let Some(entry) = entry {
            if entry.is_hosted {
                self.event_sender
                    .send(IdentityEvent::Publish { id: id.to_owned() })
                    .await
                    .unwrap();
            } else {
                let tx = self.event_sender.clone();
                let id_str = id.to_owned();
                tokio::spawn(async move {
                    sleep(Duration::from_secs(2)).await;
                    tx.send(IdentityEvent::Publish { id: id_str })
                        .await
                        .unwrap();
                });
            }
        }
    }

    pub async fn handle_post(&self, digest: &str, identity: &Identity) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        let current = state.entries.get(&identity.id).map(|entry| entry.clone());
        match current {
            None => {
                identity.verify()?;
                let entry = IdEntry::from(identity.clone());
                state.entries.insert(identity.id.clone(), entry);
                let event = IdentityEvent::Created {
                    id: identity.id.clone(),
                };
                self.event_sender.send(event).await.unwrap();
            }
            Some(entry) => {
                if digest != entry.digest {
                    entry.did.is_next(identity.clone())?;
                    let new_entry = IdEntry {
                        did: identity.clone(),
                        ..entry
                    };
                    state.entries.insert(identity.id.clone(), new_entry);
                    let event = IdentityEvent::Updated {
                        id: identity.id.clone(),
                    };
                    self.event_sender.send(event).await.unwrap();
                } else {
                    let event = IdentityEvent::Skipped {
                        id: identity.id.clone(),
                    };
                    self.event_sender.send(event).await.unwrap();
                }
            }
        }
        Ok(())
    }
}

impl IdState {
    pub fn new(ids: HashMap<String, IdEntry>) -> Self {
        IdState { entries: ids }
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

    #[tokio::test]
    async fn handle_get_test() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);
        let store = create_store(tx.clone());
        store
            .handle_get("did:p2p:bagaaierab2xsn6stgdcwfv3wvot7lboh2aewqqjfy56gzh7sibt7vxxtup4q")
            .await;
        let event = rx.recv().await;
        println!("{:?}", event);
        assert!(event.is_some());
    }

    fn create_store(tx: Sender<IdentityEvent>) -> IdStore {
        let secret_str = "beilmx4d76udjmug5ykpy657qa3pfsqbcu7fbbtuk3mgrdrxssseq";
        let secret = EdSecret::from_str(secret_str).unwrap();
        let ed_key_digest = secret.to_publickey_digest().unwrap();
        let did = Identity::new(&ed_key_digest, &ed_key_digest);
        let store = IdStore::new(HashMap::new(), tx);
        store.push_did(did);
        store
    }
}
