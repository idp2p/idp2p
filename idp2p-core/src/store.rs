use std::{collections::HashMap, sync::Mutex};

use idp2p_common::multi::keypair::Idp2pKeypair;
use tokio::sync::mpsc::Sender;

use crate::{
    identity::{self, state::IdentityState, Identity}
};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum IdStoreError {
    #[error(transparent)]
    CommandSendError(#[from] tokio::sync::mpsc::error::SendError<IdStoreOutCommand>),
    #[error(transparent)]
    EventSendError(#[from] tokio::sync::mpsc::error::SendError<IdStoreOutEvent>),
    #[error(transparent)]
    IdentityError(#[from] identity::error::IdentityError),
}
pub enum IdStoreEvent {
    ReceivedGet,
    ReceivedPost {
        last_event_id: Vec<u8>,
        did: Identity,
    },
    ReceivedMessage(Vec<u8>),
}

pub enum IdStoreCommand {
    Get(Vec<u8>),
    SendMessage { id: Vec<u8>, message: Vec<u8> },
}

#[derive(Debug)]
pub enum IdStoreOutEvent {
    IdentityCreated(Vec<u8>),
    IdentityUpdated(Vec<u8>),
    IdentitySkipped(Vec<u8>),
}

#[derive(Debug)]
pub enum IdStoreOutCommand {
    PublishGet(String),
    PublishPost { topic: String, microledger: Vec<u8> },
    PublishMessage { topic: String, message: Vec<u8> },
    WaitAndPublishPost(Vec<u8>),
}

#[derive(PartialEq, Debug, Clone)]
pub struct IdEntry {
    // Identity ledger
    pub(crate) did: Identity,
    // State of requiring publish
    pub(crate) waiting_publish: bool,
    // Current state
    pub(crate) id_state: IdentityState,
}

pub struct IdDb {
    pub(crate) id: Vec<u8>,
    pub(crate) auth_keypair: Idp2pKeypair,
    pub(crate) agree_keypair: Idp2pKeypair,
    pub(crate) identities: HashMap<Vec<u8>, IdEntry>,
}

pub struct IdStoreInput {
    pub(crate) identity: Identity,
    pub(crate) auth_keypair: Idp2pKeypair,
    pub(crate) agree_keypair: Idp2pKeypair,
    pub(crate) event_sender: Sender<IdStoreOutEvent>,
    pub(crate) command_sender: Sender<IdStoreOutCommand>,
}

pub struct IdStore {
    pub(crate) db: Mutex<IdDb>,
    pub(crate) event_sender: Sender<IdStoreOutEvent>,
    pub(crate) command_sender: Sender<IdStoreOutCommand>,
}

impl TryFrom<Identity> for IdEntry {
    type Error = IdStoreError;

    fn try_from(value: Identity) -> Result<Self, Self::Error> {
        let state = value.verify(None)?;
        let entry = IdEntry {
            waiting_publish: false,
            did: value,
            id_state: state
        };
        Ok(entry)
    }
}

impl IdDb {
    fn push_entry(&mut self, did: Identity) -> Result<(), IdStoreError> {
        let id = did.id.clone();
        let entry: IdEntry = did.try_into()?;
        self.identities.insert(id, entry);
        Ok(())
    }
}

impl IdStore {
    pub fn new(input: IdStoreInput) -> Result<Self, IdStoreError> {
        let mut db = IdDb {
            id: input.identity.id.clone(),
            auth_keypair: input.auth_keypair,
            agree_keypair: input.agree_keypair,
            identities: HashMap::new(),
        };
        db.push_entry(input.identity)?;
        let store = Self {
            db: Mutex::new(db),
            event_sender: input.event_sender,
            command_sender: input.command_sender,
        };
        Ok(store)
    }

    pub async fn handle_command(&self, cmd: IdStoreCommand) -> Result<(), IdStoreError> {
        match cmd {
            IdStoreCommand::Get(id) => {
                let topic = String::from_utf8_lossy(&id).to_string();
                let event = IdStoreOutCommand::PublishGet(topic);
                self.command_sender.send(event).await?;
            }
            IdStoreCommand::SendMessage { id, message } => {
                let topic = String::from_utf8_lossy(&id).to_string();
                //let msg = IdMessage::new(to, body)?;
                let event = IdStoreOutCommand::PublishMessage{topic: topic, message: vec![]};
                self.command_sender.send(event).await?;
            }
        }
        Ok(())
    }

    pub async fn handle_event(&self, topic: &str, event: IdStoreEvent) -> Result<(), IdStoreError> {
        let mut db = self.db.lock().unwrap();
        match event {
            IdStoreEvent::ReceivedGet => {
                let entry = db.identities.get_mut(topic.as_bytes());
                if let Some(entry) = entry {
                    if &entry.did.id == topic.as_bytes() {
                        log::info!("Published id: {:?}", topic);
                        let cmd = IdStoreOutCommand::PublishPost {
                            topic: topic.to_string(),
                            microledger: entry.did.microledger.clone(),
                        };
                        self.command_sender.send(cmd).await?;
                    } else {
                        entry.waiting_publish = true;
                        let cmd = IdStoreOutCommand::WaitAndPublishPost(topic.as_bytes().to_vec());
                        self.command_sender.send(cmd).await?;
                    }
                }
            }
            IdStoreEvent::ReceivedPost { last_event_id, did } => {
                let mut state = self.db.lock().unwrap();
                let current = db.identities.get_mut(topic.as_bytes());
                match current {
                    // When identity is new
                    None => {
                        did.verify(None)?;
                        let entry = did.clone().try_into()?;
                        state.identities.insert(did.id.clone(), entry);
                        log::info!("Got id: {:?}", did.id);
                        let event = IdStoreOutEvent::IdentityCreated(topic.as_bytes().to_vec());
                        self.event_sender.send(event).await?;
                    }
                    // There is a current identity
                    Some(entry) => {
                        // If there is a waiting publish, remove it
                        entry.waiting_publish = false;
                        // Identity has a new state
                        if last_event_id != entry.id_state.last_event_id {
                            //entry.did.is_next(did.clone())?;
                            entry.did = did.clone();
                            log::info!("Updated id: {:?}", did.id);
                            let event = IdStoreOutEvent::IdentityUpdated(topic.as_bytes().to_vec());
                            self.event_sender.send(event).await?;
                        } else {
                            log::info!("Skipped id: {:?}", did.id);
                            let event = IdStoreOutEvent::IdentitySkipped(topic.as_bytes().to_vec());
                            self.event_sender.send(event).await?;
                        }
                    }
                }
            }
            IdStoreEvent::ReceivedMessage(msg) => {
                let entry = db.identities.get_mut(topic.as_bytes());
                if let Some(entry) = entry {

                }
                todo!()
            },
        }
        Ok(())
    }
}
