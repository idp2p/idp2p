use crate::behaviour::IdentityEvent;
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
    pub sender: Sender<IdentityEvent>,
}

#[derive(PartialEq, Debug)]
pub struct IdState {
    pub entries: HashMap<String, IdEntry>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct IdEntry {
    pub is_hosted: bool,
    pub digest: String,
    pub did: Identity,
    pub last_updated: i64,
}

impl IdStore {
    pub fn new(sender: Sender<IdentityEvent>) -> IdStore {
        let state = IdState {
            entries: HashMap::new(),
        };
        let shared = IdShared {
            state: Mutex::new(state),
            sender: sender,
        };
        IdStore {
            shared: Arc::new(shared),
        }
    }

    pub fn get(&self, id: &str) -> Option<IdEntry> {
        let state = self.shared.state.lock().unwrap();
        state.entries.get(id).map(|entry| entry.clone())
    }

    pub fn put(&self, id: &str, entry: IdEntry) {
        let mut state = self.shared.state.lock().unwrap();
        state.entries.insert(id.to_owned(), entry);
    }

    pub fn publish_event(&self, event: IdentityEvent) {
        self.shared
            .sender
            .try_send(event)
            .expect("Couldn't send event");
    }
}

impl IdEntry {
    pub fn new(did: Identity) -> Self {
        IdEntry {
            digest: did.get_digest(),
            last_updated: Utc::now().timestamp(),
            is_hosted: true,
            did: did,
        }
    }

    pub fn from(did: Identity) -> Self {
        IdEntry {
            digest: did.get_digest(),
            last_updated: Utc::now().timestamp(),
            is_hosted: false,
            did: did,
        }
    }
}
