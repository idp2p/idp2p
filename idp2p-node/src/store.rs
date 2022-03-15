use crate::IdentityEvent;
use idp2p_common::anyhow::Result;
use idp2p_common::chrono::Utc;
use idp2p_core::did::Identity;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc::Sender;

pub struct IdStore {
    pub shared: Arc<IdShared>,
}

pub struct IdShared {
    pub state: Mutex<IdState>,
    pub tx: Sender<IdentityEvent>,
    pub owner: String,
}

#[derive(PartialEq, Debug, Clone)]
pub struct IdState {
    pub entries: HashMap<String, IdEntry>,
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
    pub fn new(tx: Sender<IdentityEvent>, owner: Identity) -> IdStore {
        let state = IdState {
            entries: HashMap::new(),
        };
        let shared = Arc::new(IdShared {
            state: Mutex::new(state),
            owner: owner.id.clone(),
            tx: tx,
        });
        tokio::spawn(listen_events(shared.clone()));
        IdStore { shared: shared }
    }

    pub fn handle_get(&self, id: &str) {
        let state = self.shared.state.lock().unwrap();
        let entry = state.entries.get(id).map(|entry| entry.clone());
        if let Some(entry) = entry {
            if entry.is_hosted {
                // add queue to publish
                // return Ok(PublishCommand{did: entry.did, is_hosted: true});
            } else {
                // add to queue to publish
                // to do()
            }
        }
    }

    pub fn handle_post(&self, digest: &str, identity: &Identity) -> Result<()> {
        let mut state = self.shared.state.lock().unwrap();
        let current = state.entries.get(&identity.id).map(|entry| entry.clone());
        match current {
            None => {
                identity.verify()?;
                let entry = IdEntry::from(identity.clone());
                state.entries.insert(identity.id.clone(), entry);
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
                }else{

                }
            }
        }
        let event = IdentityEvent::Skipped {
            id: identity.id.clone(),
        };
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

async fn listen_events(shared: Arc<IdShared>) {
    let _ = shared.tx.send(IdentityEvent::Skipped { id: "".to_owned() }).await;
    //shared.tx.send()
}
