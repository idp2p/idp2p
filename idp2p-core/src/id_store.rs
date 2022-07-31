use std::{collections::HashMap, sync::Mutex};

use idp2p_common::multi::{
    agreement_secret::Idp2pAgreementSecret, id::Idp2pCodec, key_secret::Idp2pKeySecret,
    message::Idp2pMessage,
};
use tokio::sync::mpsc::Sender;

use crate::{
    error::Idp2pError, id_message::IdMessage, id_state::IdentityState, identity::Identity,
    HandlerResolver,
};

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
    pub(crate) owner: IdentityState,
    pub(crate) auth_secret: Idp2pKeySecret,
    pub(crate) agree_secret: Idp2pAgreementSecret,
    pub(crate) identities: HashMap<Vec<u8>, IdEntry>,
}

pub struct IdStoreOptions {
    pub(crate) owner: Identity,
    pub(crate) auth_secret: Idp2pKeySecret,
    pub(crate) agree_secret: Idp2pAgreementSecret,
    pub(crate) event_sender: Sender<IdStoreOutEvent>,
    pub(crate) command_sender: Sender<IdStoreOutCommand>,
}

pub struct IdStore {
    pub(crate) db: Mutex<IdDb>,
    pub(crate) event_sender: Sender<IdStoreOutEvent>,
    pub(crate) command_sender: Sender<IdStoreOutCommand>,
    pub(crate) codec: Idp2pCodec,
}

impl IdDb {
    fn push_entry(&mut self, did: Identity) -> Result<(), Idp2pError> {
        let id = did.id.clone();
        let id_state = did.verify(None)?;
        let entry = IdEntry {
            waiting_publish: false,
            did: did,
            id_state: id_state,
        };
        self.identities.insert(id, entry);
        Ok(())
    }
}

impl IdStore {
    pub fn new(options: IdStoreOptions) -> Result<Self, Idp2pError> {
        let owner_state = options.owner.verify(None)?;
        let db = IdDb {
            owner: owner_state,
            auth_secret: options.auth_secret,
            agree_secret: options.agree_secret,
            identities: HashMap::new(),
        };
        let store = Self {
            db: Mutex::new(db),
            event_sender: options.event_sender,
            command_sender: options.command_sender,
            codec: Idp2pCodec::Protobuf,
        };
        Ok(store)
    }

    pub async fn handle_command(&self, cmd: IdStoreCommand) -> Result<(), Idp2pError> {
        let db = self.db.lock().unwrap();
        match cmd {
            IdStoreCommand::Get(id) => {
                let topic = String::from_utf8_lossy(&id).to_string();
                let event = IdStoreOutCommand::PublishGet(topic);
                self.command_sender.send(event).await?;
            }
            IdStoreCommand::SendMessage { id, message } => {
                let topic = String::from_utf8_lossy(&id).to_string();
                let from = db
                    .identities
                    .get(&db.owner.id.clone())
                    .expect("")
                    .id_state
                    .clone();
                let to = db.identities.get(&id).expect("").id_state.clone();
                let msg = self.codec.resolve_msg_handler().seal_msg(
                    db.auth_secret.clone(),
                    from,
                    to,
                    &message,
                )?;
                let event = IdStoreOutCommand::PublishMessage {
                    topic: topic,
                    message: msg,
                };
                self.command_sender.send(event).await?;
            }
        }
        Ok(())
    }

    pub async fn handle_event(&self, topic: &str, event: IdStoreEvent) -> Result<(), Idp2pError> {
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
                let current = db.identities.get_mut(topic.as_bytes());
                let id = did.id.clone();
                log::info!("Got id: {:?}", id);
                match current {
                    // When identity is new
                    None => {
                        db.push_entry(did)?;
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
                    let msg = Idp2pMessage::from_bytes(&msg)?;
                    let dec_msg = msg
                        .id
                        .codec
                        .resolve_msg_handler()
                        .decode_msg(db.agree_secret.clone(), &msg.body)?;
                    let from = db
                        .identities
                        .get(&dec_msg.from)
                        .ok_or(Idp2pError::RequiredField("From".to_string()))?;
                }
                todo!()
            }
        }
        Ok(())
    }
}
