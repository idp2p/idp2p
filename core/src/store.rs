use std::{collections::HashMap, sync::Mutex};

use tokio::sync::mpsc::Sender;

use crate::{
    identity::{error::IdentityError, state::IdentityState, Identity},
    multi::hash::Idp2pHash, message::IdMessage,
};

use super::Idp2pConfig;

pub enum IdentityEvent {
    ReceivedGet(Vec<u8>),
    ReceivedPost(Identity),
    ReceivedMessage{topic: String, msg: IdMessage},
}

pub enum IdentityCommand {
    Get(Vec<u8>),
    SendMessage,
}

pub enum IdentityOutEvent {
    IdentityCreated,
    IdentityUpdated,
    IdentitySkipped,
}

pub enum IdentityOutCommand {
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
    pub id_state: IdentityState,
}

pub struct State {
    id: Vec<u8>,
    auth_keypair: Idp2pKeypair,
    agree_keypair: Idp2pKeypair,
    identities: HashMap<Vec<u8>, IdEntry>
}

pub struct IdStoreInput {
    pub(crate) identity: Identity,
    pub(crate) auth_keypair: Idp2pKeypair,
    pub(crate) agree_keypair: Idp2pKeypair,
    pub(crate) event_sender: Sender<IdentityOutEvent>,
    pub(crate) command_sender: Sender<IdentityOutCommand>,
}

pub struct IdStore {
    pub state: Mutex<State>,
    event_sender: Sender<IdentityOutEvent>,
    command_sender: Sender<IdentityOutCommand>,
}

impl TryFrom<Identity> for IdEntry {
    type Error = IdentityError;

    fn try_from(value: Identity) -> Result<Self, Self::Error> {
        let mh = Code::Sha256.digest(&did.microledger);
        /*let entry = IdEntry {
            digest: mh.to_bytes(),
            require_publish: false,
            did: did,
        };
        entry*/
        todo!()
    }
}

impl IdState {
    fn push_entry(&mut self, did: Identity) {
        let id = did.id.clone();
        let entry: IdEntry = did.into();
        self.identities.insert(id, entry);
    }
}

impl IdStore {
    pub fn new(input: IdStoreInput) -> Self {
        let mut state = IdState {
            config: config,
            identities: HashMap::new(),
        };
        state.push_entry(did);
        let store = Self {
            state: Mutex::new(state),
            event_sender: input.event_sender,
            command_sender: input.command_sender
        };
        store
    }

    pub async fn handle_command(&self, cmd: IdentityCommand) -> Result<(), String> {
        match cmd {
            IdentityCommand::Get(id) => {
                // init a get request
                // tx.try_send()
            }
            IdentityCommand::SendMessage => {
                todo!()
            }
        }
        Ok(())
    }

    pub async fn handle_event(&self, event: IdentityEvent) -> Result<(), String> {
        match event {
            IdentityEvent::ReceivedGet(id) => {
                let mut state = self.state.lock()?;
                let entry = state.identities.get_mut(&id);
                if let Some(entry) = entry {
                    if &entry.did.id == id {
                        log::info!("Published id: {:?}", id);
                        return Ok(HandleGetResult::Publish(id.to_vec()));
                    } else {
                        entry.require_publish = true;
                        let e = HandleGetResult::WaitAndPublish(id.to_vec());
                        self.command_sender.try_send(e).await?;
                    }
                }
                Ok(())
            }
            IdentityEvent::ReceivedPost(did) => {
                let mut state = self.state.lock().unwrap();
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
            IdentityEvent::ReceivedMessage => todo!(),
        }
        Ok(())
    }
}

/*
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
    let hasher = state.config.hash.clone();
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
}*/
