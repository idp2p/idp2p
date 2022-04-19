use idp2p_common::anyhow::Result;
use idp2p_common::log;
use idp2p_core::did::identity::Identity;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

pub enum HandlePostResult {
    Created,
    Updated,
    Skipped,
}

pub enum HandleGetResult {
    Publish(String),
    WaitAndPublish(u8),
}

pub struct NodeStore {
    pub state: Mutex<NodeState>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct NodeState {
    pub identities: HashMap<String, IdEntry>,
    pub clients: HashMap<String, Client>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct IdEntry {
    // Hash of identity for check
    pub digest: String,
    // Identity info
    pub did: Identity,
    pub require_publish: bool,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Client {
    pub id: String,
    pub received_messages: Vec<String>,
    pub subscriptions: HashSet<String>,
}

impl NodeStore {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(NodeState::new()),
        }
    }

    /// Get identity for post event
    pub fn get_for_post(&self, id: &str) -> Option<Identity> {
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
    pub fn get_did(&self, id: &str) -> Option<Identity> {
        let state = self.state.lock().unwrap();
        let entry = state.identities.get(id).map(|entry| entry.clone());
        if let Some(entry) = entry {
            return Some(entry.did);
        }
        None
    }

    pub async fn create(&self, did: Identity) {
        let mut state = self.state.lock().unwrap();
        let id = did.id.clone();
        let entry = did.into();
        state.identities.insert(id.clone(), entry);
    }

    /// Handle identity get event
    pub async fn handle_get(&self, id: &str) -> Result<HandleGetResult> {
        let mut state = self.state.lock().unwrap();
        let is_client = state.clients.get(id).is_some();
        let entry = state.identities.get_mut(id);
        if let Some(entry) = entry {
            if is_client {
                log::info!("Published id: {}", id);
                return Ok(HandleGetResult::Publish(id.to_owned()));
            } else {
                entry.require_publish = true;
                return Ok(HandleGetResult::WaitAndPublish(2));
            }
        }
        idp2p_common::anyhow::bail!("Identity is not stored")
    }

    /// Handle identity post event
    pub async fn handle_post(&self, digest: &str, identity: &Identity) -> Result<HandlePostResult> {
        let mut state = self.state.lock().unwrap();
        let current = state.identities.get_mut(&identity.id);
        match current {
            // When incoming identity is new
            None => {
                identity.verify()?;
                let entry = IdEntry::from(identity.clone());
                state.identities.insert(identity.id.clone(), entry);
                log::info!("Got id: {}", identity.id);
                return Ok(HandlePostResult::Created);
            }
            // There is a current identity
            Some(entry) => {
                // If there is a waiting publish, remove it
                entry.require_publish = false;
                // Identity has a new state
                if digest != entry.digest {
                    entry.did.is_next(identity.clone())?;
                    entry.did = identity.clone();
                    log::info!("Updated id: {}", identity.id);
                    return Ok(HandlePostResult::Updated);
                } else {
                    log::info!("Skipped id: {}", identity.id);
                    return Ok(HandlePostResult::Skipped);
                }
            }
        }
    }
}

impl NodeState {
    pub fn new() -> Self {
        NodeState {
            identities: HashMap::new(),
            clients: HashMap::new(),
        }
    }
}

impl From<Identity> for IdEntry {
    fn from(did: Identity) -> Self {
        IdEntry {
            digest: did.get_digest(),
            require_publish: false,
            did: did,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use idp2p_common::ed_secret::EdSecret;

    #[tokio::test]
    async fn handle_get_test() -> Result<()> {
        let store = create_store().await;
        let id = "did:p2p:bagaaierab2xsn6stgdcwfv3wvot7lboh2aewqqjfy56gzh7sibt7vxxtup4q";
        let result = store.handle_get(id).await?;
        matches!(result, HandleGetResult::WaitAndPublish(2));
        Ok(())
    }

    async fn create_store() -> NodeStore {
        let secret_str = "beilmx4d76udjmug5ykpy657qa3pfsqbcu7fbbtuk3mgrdrxssseq";
        let secret = EdSecret::from_str(secret_str).unwrap();
        let ed_key_digest = secret.to_publickey_digest().unwrap();
        let did = Identity::new(&ed_key_digest, &ed_key_digest);
        let store = NodeStore::new();
        store.create(did).await;
        store
    }
}
