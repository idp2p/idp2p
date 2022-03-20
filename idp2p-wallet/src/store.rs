use idp2p_common::anyhow::Result;
use idp2p_common::chrono::Utc;
use idp2p_core::did::Identity;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;
use tokio::sync::mpsc::Sender;

pub struct WalletStore {
    base_path: String,
    shared: Arc<WalletShared>,
}

pub struct WalletShared {
    wallet: Mutex<Wallet>
}

#[derive(PartialEq, Debug, Clone)]
pub struct Wallet {
    #[serde(with = "encode_vec")]
    pub salt: Vec<u8>,
    #[serde(with = "encode_vec")]
    pub iv: Vec<u8>,
    #[serde(with = "encode_vec")]
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ciphertext: Vec<u8>,
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
        let mut state = IdState {
            entries: HashMap::new(),
            events: BTreeMap::new(),
        };
        let shared = Arc::new(IdShared {
            state: Mutex::new(state),
            owner: owner,
            tx: tx,
        });
        tokio::spawn(listen_events(shared.clone()));
        IdStore { shared: shared }
    }

    pub fn get_did(&self, id: &str) -> Identity {
        let state = self.shared.state.lock().unwrap();
        let entry = state.entries.get(id).map(|entry| entry.clone());
        entry.unwrap().did
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
    
}
