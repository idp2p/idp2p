use crate::IdentityEvent;
use idp2p_common::anyhow::Result;
use idp2p_common::chrono::Utc;
use idp2p_core::did::Identity;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;
use tokio::sync::mpsc::Sender;

pub struct IdStoreOptions {
    pub event_sender: Sender<IdentityEvent>,
    pub entries: HashMap<String, IdEntry>,
}
pub struct IdStore {
    pub shared: Arc<IdShared>,
}

pub struct IdShared {
    pub state: Mutex<IdState>,
    event_sender: Sender<IdentityEvent>
}

#[derive(PartialEq, Debug, Clone)]
pub struct IdState {
    pub entries: HashMap<String, IdEntry>,
    events: BTreeMap<Instant, String>,
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
    pub fn new(options: IdStoreOptions) -> IdStore {
        let state = IdState {
            entries: options.entries,
            events: BTreeMap::new(),
        };
        let shared = Arc::new(IdShared {
            state: Mutex::new(state),
            event_sender: options.event_sender,
        });
        tokio::spawn(listen_events(shared.clone()));
        IdStore { shared: shared }
    }

    pub fn get_did(&self, id: &str) -> Identity {
        let state = self.shared.state.lock().unwrap();
        let entry = state.entries.get(id).map(|entry| entry.clone());
        entry.unwrap().did
    }

    pub fn push_did(&self, did: Identity) {
        let mut state = self.shared.state.lock().unwrap();
        let id = did.id.clone();
        let entry = IdEntry::new(did);
        state.entries.insert(id, entry);
    }

    pub async fn handle_get(&self, id: &str) {
        let state = self.shared.state.lock().unwrap();
        let entry = state.entries.get(id).map(|entry| entry.clone());
        if let Some(entry) = entry {
            if entry.is_hosted {
                let event = IdentityEvent::Published { id: id.to_owned() };
                self.shared.event_sender.send(event).await.unwrap();
            } else {
                // add queue to publish
                // to do()
            }
        }
    }

    pub async fn handle_post(&self, digest: &str, identity: &Identity) -> Result<()> {
        let mut state = self.shared.state.lock().unwrap();
        let current = state.entries.get(&identity.id).map(|entry| entry.clone());
        match current {
            None => {
                identity.verify()?;
                let entry = IdEntry::from(identity.clone());
                state.entries.insert(identity.id.clone(), entry);
                let event = IdentityEvent::Created {
                    id: identity.id.clone(),
                };
                self.shared.event_sender.send(event).await.unwrap();
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
                    self.shared.event_sender.send(event).await.unwrap();
                }
            }
        }
        Ok(())
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

async fn listen_events(_shared: Arc<IdShared>) {
    //println!("To do owner: {}", shared.owner.id);
    /*let _ = shared
    .tx
    .send(IdentityEvent::Published { id: "".to_owned() })
    .await;*/
    //shared.tx.send()
}
