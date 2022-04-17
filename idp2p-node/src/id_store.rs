use idp2p_common::anyhow::Result;
use idp2p_common::chrono::Utc;
use idp2p_common::log;
use idp2p_core::did::identity::Identity;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use tokio::sync::mpsc::Sender;
use tokio::time::{sleep, Duration};

use idp2p_core::protocol::id_message::IdentityMessage;
use crate::IdentityStoreEvent;

pub struct IdNodeStore {
    pub state: Mutex<IdNodeState>,
    pub event_sender: Sender<IdentityStoreEvent>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct IdNodeState {
    pub identities: HashMap<String, IdEntry>,
    pub clients: HashMap<String, Client>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct IdEntry {
    pub digest: String,
    pub did: Identity,
    pub last_updated: i64,
    pub last_published: i64,
    pub is_hosted: bool,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Client {
    pub id: String,
    pub authentication_index: u32,
    pub ciphertext: Vec<u8>,
    pub sent_messages: Vec<String>,
    pub received_messages: Vec<String>,
    pub subscriptions: HashSet<String>,
}

impl IdNodeStore {
    pub fn new(es: Sender<IdentityStoreEvent>) -> Self {
        Self {
            state: Mutex::new(IdNodeState::new()),
            event_sender: es,
        }
    }

    pub fn get_did(&self, id: &str) -> Identity {
        let state = self.state.lock().unwrap();
        let entry = state.identities.get(id).map(|entry| entry.clone());
        entry.unwrap().did
    }

    pub fn get_message(&self, id: &str) -> Option<IdentityMessage> {
        let state = self.state.lock().unwrap();
        let entry = state.identities.get(id).map(|entry| entry.clone());
        if let Some(entry) = entry {
            return Some(IdentityMessage::new_post(entry.did));
        }
        None
    }

    pub async fn create_did(&self, did: Identity) {
        let mut state = self.state.lock().unwrap();
        let id = did.id.clone();
        let entry = IdEntry::new(did);
        state.identities.insert(id.clone(), entry);
        let event = IdentityStoreEvent::Created { id };
        self.event_sender.send(event).await.unwrap();
    }

    pub async fn handle_get(&self, id: &str) {
        let state = self.state.lock().unwrap();
        let entry = state.identities.get(id).map(|entry| entry.clone());
        if let Some(entry) = entry {
            if entry.is_hosted {
                self.event_sender
                    .send(IdentityStoreEvent::GetHandled { id: id.to_owned() })
                    .await
                    .unwrap();
                log::info!("Published id: {}", id);
            } else {
                let tx = self.event_sender.clone();
                let id_str = id.to_owned();
                tokio::spawn(async move {
                    sleep(Duration::from_secs(2)).await;
                    tx.send(IdentityStoreEvent::GetHandled { id: id_str })
                        .await
                        .unwrap();
                });
            }
        }
    }

    pub async fn handle_post(&self, digest: &str, identity: &Identity) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        let current = state
            .identities
            .get(&identity.id)
            .map(|entry| entry.clone());
        match current {
            None => {
                identity.verify()?;
                let entry = IdEntry::from(identity.clone());
                state.identities.insert(identity.id.clone(), entry);
                let event = IdentityStoreEvent::PostHandled {
                    id: identity.id.clone(),
                };
                self.event_sender.send(event).await.unwrap();
                log::info!("Got id: {}", identity.id);
            }
            Some(entry) => {
                if digest != entry.digest {
                    entry.did.is_next(identity.clone())?;
                    let new_entry = IdEntry {
                        did: identity.clone(),
                        ..entry
                    };
                    state.identities.insert(identity.id.clone(), new_entry);
                    let event = IdentityStoreEvent::PostHandled {
                        id: identity.id.clone(),
                    };
                    self.event_sender.send(event).await.unwrap();
                    log::info!("Updated id: {}", identity.id);
                } else {
                    log::info!("Skipped id: {}", identity.id);
                }
            }
        }
        Ok(())
    }

    pub async fn handle_jwm(&self) -> Result<()> {
        //
        Ok(())
    }
}

impl IdNodeState {
    pub fn new() -> Self {
        IdNodeState {
            identities: HashMap::new(),
            clients: HashMap::new(),
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
}

impl From<Identity> for IdEntry {
    fn from(did: Identity) -> Self {
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
        let store = create_store(tx.clone()).await;
        store
            .handle_get("did:p2p:bagaaierab2xsn6stgdcwfv3wvot7lboh2aewqqjfy56gzh7sibt7vxxtup4q")
            .await;
        let event = rx.recv().await;
        println!("{:?}", event);
        assert!(event.is_some());
    }

    async fn create_store(tx: Sender<IdentityStoreEvent>) -> IdNodeStore {
        let secret_str = "beilmx4d76udjmug5ykpy657qa3pfsqbcu7fbbtuk3mgrdrxssseq";
        let secret = EdSecret::from_str(secret_str).unwrap();
        let ed_key_digest = secret.to_publickey_digest().unwrap();
        let did = Identity::new(&ed_key_digest, &ed_key_digest);
        let store = IdNodeStore::new(tx);
        store.create_did(did).await;
        store
    }
}
