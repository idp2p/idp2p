
pub enum HandlePostResult {
    Created,
    Updated,
    Skipped,
}

pub enum HandleGetResult {
    Publish(String),
    WaitAndPublish(String),
}

#[derive(PartialEq, Debug, Clone)]
pub struct IdEntry {
    // Hash of identity for check
    pub digest: Vec<u8>,
    // Identity info
    pub did: Identity,
    pub require_publish: bool,
}

pub struct IdNodeState {
    authentication_secret: Idp2pSecret,
    agreement_secret: Idp2pSecret,
    identity: IdEntry,
    identities: HashMap<Vec<u8>, IdEntry>
}

impl IdNodeState {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(NodeState::new()),
        }
    }

    /// Get identity for post event
    pub fn get_did_for_post(&self, id: &str) -> Option<Identity> {
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
                return Ok(HandleGetResult::WaitAndPublish(id.to_owned()));
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