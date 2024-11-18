use std::{collections::HashMap, sync::Arc};

use anyhow::{Ok, Result};
use cid::Cid;
use idp2p_common::{cbor::decode, content::Content};
use idp2p_id::{event::PersistedIdEvent, PersistedId};
use serde::{Deserialize, Serialize};
use libp2p::gossipsub::{self, Event as GossipsubEvent};
use tokio::sync::oneshot;
use wasmtime::{Engine, Module};
use crate::{entry::IdEntry, store::KvStore};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdMessageRequest(String);
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdMessageResponse(Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdGossipMessageKind {
    // Resolve identity
    Resolve,
    // Provide an identity document
    Provide { id: PersistedId },
    // Notify an identity event
    NotifyEvent { event: PersistedIdEvent },
    // Notify message
    NotifyMessage { id: Cid, providers: Vec<String> },
}

pub struct IdMessageHandler {
    engine: Engine,
    kv_store: Arc<dyn KvStore>,
    modules: HashMap<String, Module>,    
    event_sender: oneshot::Sender<String>
}

impl IdMessageHandler {
    pub fn handle(&self, event: GossipsubEvent) -> anyhow::Result<()>{
        match event {
            GossipsubEvent::Message {
                propagation_source: _,
                message_id: _,
                message,
            } => {
                let content = Content::from_bytes(message.data.as_slice())?;
                let msg: IdGossipMessageKind = decode(&content.payload)?;
                let id_key = format!("/identities/{}", message.topic.as_str());
                if let Some(id_entry) = self.kv_store.get(&id_key)?{
                    let id_entry: IdEntry = decode(&id_entry)?;
                    match &msg {
                        IdGossipMessageKind::Resolve => {
                            if id_entry.provided {
                                //behaviour.publish(message.topic, b"data")?;
                            }
                        },
                        IdGossipMessageKind::NotifyEvent { event } => {},
                        IdGossipMessageKind::NotifyMessage { id, providers } => {},
                        _ => {}
                    }
                }else {
                    match &msg {
                        IdGossipMessageKind::Provide {id} => {
                             //id_verifier::verify_inception(version, payload)
                        }
                        _ => {
                           
                        }
                    }
                }
                // read 4 bytes from the message
                // get id entry from the store
                // get module from the hashmap with the version
                // call handle-message with the message and id entry
                // get the response from the handle-message
                // set the response in the store and/or publish it
                // commit the store
                
                
                /*let id_component: Vec<u8> = id_store
                    .get(format!("/components/{}", content.version).as_str())?
                    .unwrap();
                let component = wasmtime::component::Component::from_binary(&engine, &p2p_component)?;
                let id_components: HashMap<String, Component> = HashMap::new();
                let mut store = wasmtime::Store::new(&engine, IdState {});
                let linker = wasmtime::component::Linker::new(&engine);
                let (idp2p, instance) =
                    Idp2pP2p::instantiate(&mut store, &components.get("k").unwrap(), &linker)?;*/
            }
            _ => {}
        }
       Ok(())
    }
}
// kv store
// modules
// 
use libp2p::gossipsub::Behaviour;

impl IdGossipMessageKind {
    pub fn handle<S: KvStore>(&self, gossipsub: &Behaviour, store: &S) -> Result<()> {
        Ok(())
    }
}

/*pub fn handle_message(request:IdRequest) -> anyhow::Result<IdResponse> {
    let msg: IdGossipMessageKind = decode(&request.message)?;
    if let Some(entry) = request.id_entry {
        let entry: IdEntry = decode(&entry)?;
        match &msg {
            IdGossipMessageKind::Resolve => {
                if entry.provided {
                    return Ok(IdResponse { update: None, publish: None });
                }
            },
            IdGossipMessageKind::Provide { doc } => {},
            IdGossipMessageKind::NotifyEvent { event } => {},
            IdGossipMessageKind::NotifyMessage { id, providers } => {},
        }
    }else {
        match &msg {
            IdGossipMessageKind::Provide { doc } => {
                 //id_verifier::verify_inception(version, payload)
            }
            _ => {
                return Ok(IdResponse { update: None, publish: None });
            }
        }
    }
    todo!()
}*/
