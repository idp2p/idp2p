use std::{collections::HashMap, sync::Mutex};

use crate::{
    identity::{error::IdentityError, Identity},
    multi::hash::Idp2pHash,
};

use super::Idp2pConfig;

pub enum HandlePostResult {
    Created,
    Updated,
    Skipped,
}

pub enum HandleGetResult {
    Publish(Vec<u8>),
    WaitAndPublish(Vec<u8>),
}

#[derive(PartialEq, Debug, Clone)]
pub struct IdEntry {
    pub did: Identity,
    // Digest of microledger. It is useful for quick checking up-to-date
    pub digest: Vec<u8>,
    // State of requiring publish
    pub require_publish: bool,
}

pub struct IdState {
    config: Idp2pConfig,
    identities: HashMap<Vec<u8>, IdEntry>,
}

pub struct IdStore {
    pub state: Mutex<IdState>,
}

impl IdStore {
    pub fn new(config: Idp2pConfig, did: Identity) -> Self {
        let store = Self {
            state: Mutex::new(IdState {
                config: config,
                identities: HashMap::new(),
            }),
        };
        store.create(did);
        store
    }

    pub fn create(&self, did: Identity) {
        let mut state = self.state.lock().unwrap();
        let id = did.id.clone();
        let entry: IdEntry = did.into();
        state.identities.insert(id, entry);
    }

    /// Get identity for post event
    pub fn get_did_for_post(&self, id: &[u8]) -> Option<Identity> {
        let state = self.state.lock().unwrap();
        let entry = state.identities.get(id);
        if let Some(entry) = entry {
            // Check if entry is not published or received
            if entry.require_publish {
                return Some(entry.did.clone());
            }
        }
        None
    }

    /// Get identity by id
    pub fn get_did(&self, id: &[u8]) -> Option<Identity> {
        let state = self.state.lock().unwrap();
        let entry = state.identities.get(id).map(|entry| entry.clone());
        if let Some(entry) = entry {
            return Some(entry.did);
        }
        None
    }

    /// Handle identity get event
    pub async fn handle_get(&self, id: &[u8]) -> Result<HandleGetResult, IdentityError> {
        let mut state = self.state.lock().unwrap();
        let entry = state.identities.get_mut(id);
        if let Some(entry) = entry {
            if &entry.did.id == id {
                log::info!("Published id: {:?}", id);
                return Ok(HandleGetResult::Publish(id.to_vec()));
            } else {
                entry.require_publish = true;
                return Ok(HandleGetResult::WaitAndPublish(id.to_vec()));
            }
        }
        Err(IdentityError::Other)
    }

    /// Handle identity post event
    pub async fn handle_post(&self, did: &Identity) -> Result<HandlePostResult, IdentityError> {
        let mut state = self.state.lock().unwrap();
        let hash = state.config.hash.clone();
        let current = state.identities.get_mut(&did.id);
        match current {
            // When incoming identity is new
            None => {
                did.verify()?;
                let entry = did.clone().into();
                state.identities.insert(did.id.clone(), entry);
                log::info!("Got id: {:?}", did.id);
                return Ok(HandlePostResult::Created);
            }
            // There is a current identity
            Some(entry) => {
                let digest = hash.digest(&did.microledger);
                // If there is a waiting publish, remove it
                entry.require_publish = false;
                // Identity has a new state
                if digest.to_bytes() != entry.digest {
                    //entry.did.is_next(did.clone())?;
                    entry.did = did.clone();
                    log::info!("Updated id: {:?}", did.id);
                    return Ok(HandlePostResult::Updated);
                } else {
                    log::info!("Skipped id: {:?}", did.id);
                    return Ok(HandlePostResult::Skipped);
                }
            }
        }
    }
}

impl From<Identity> for IdEntry {
    fn from(did: Identity) -> Self {
        let mh = Idp2pHash::default().digest(&did.microledger);
        let entry = IdEntry {
            digest: mh.to_bytes(),
            require_publish: false,
            did: did,
        };
        entry
    }
}
