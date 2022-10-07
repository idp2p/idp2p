use std::{
    collections::{HashMap, VecDeque},
    sync::Mutex,
};

use idp2p_common::multi::base::Idp2pBase;
use idp2p_core::{id_state::IdentityState, identity::Identity};
use libp2p::{Multiaddr, PeerId};

use crate::error::Idp2pGossipError;

type Id = Vec<u8>;

#[derive(PartialEq, Debug, Clone)]
pub enum GossipStoreInEvent {
    GossipResolve,
    GossipMutation {
        last_event_id: Vec<u8>,
        did: Identity,
    },
    GossipMessage(Vec<u8>),
    PairRequest(),
}

#[derive(PartialEq, Debug, Clone)]
pub enum GossipStoreOutEvent {
    // Gossipsub publish
    Publish { topic: String, microledger: Vec<u8> },
    // Wait 2 sec, try to publish
    WaitAndPublish(Id),
    // Notify identity pairs
    NotifyId(Id),
    // Notify all subscribers
    NotifySubscribers(Id),
}

#[derive(PartialEq, Debug, Clone)]
pub enum IdentityEvent {
    ResolvedId(Id),
    MutatedId(Id),
    ReceivedMessage,
}

#[derive(PartialEq, Debug, Clone)]
pub struct IdPeer {
    peer_id: PeerId,
    multiaddr: Multiaddr,
}

#[derive(PartialEq, Debug, Clone)]
pub struct IdEntry {
    // Identity ledger
    did: Identity,
    // State of requiring publish
    waiting_publish: bool,
    // Current state
    id_state: IdentityState,
    // Subscribers
    subscribers: Vec<Id>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct IdPair {
    token: String,
    // Mutation, Resolve, Message
    events: Vec<IdentityEvent>,
    // Pairs
    peers: Vec<IdPeer>,
}

pub struct GossipState {
    pairs: HashMap<Id, IdPair>,
    identities: HashMap<Id, IdEntry>,
    events: VecDeque<GossipStoreOutEvent>,
}

pub struct GossipStore {
    state: Mutex<GossipState>,
}

impl IdEntry {
    pub fn new(did: Identity) -> Result<Self, Idp2pGossipError> {
        let id_state = did.verify(None)?;
        Ok(Self {
            waiting_publish: false,
            did: did,
            id_state: id_state,
            subscribers: vec![]
        })
    }
}

impl IdPair {
    pub fn new(token: &str) -> Self {
        Self {
            token: token.to_owned(),
            events: vec![],
            peers: vec![],
        }
    }
}
impl GossipStore {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(GossipState {
                pairs: HashMap::new(),
                identities: HashMap::new(),
                events: VecDeque::new(),
            }),
        }
    }

    pub fn handle_gossip_event(
        &self,
        topic: &str,
        event: GossipStoreInEvent,
    ) -> Result<(), Idp2pGossipError> {
        let id = Idp2pBase::decode(topic)?;
        let mut db = self.state.lock().unwrap();
        match event {
            GossipStoreInEvent::GossipResolve => {
                let is_pair = db.pairs.contains_key(&id);
                let entry = db.identities.get_mut(&id);
                if let Some(entry) = entry {
                    if is_pair {
                        log::info!("Published id: {:?}", topic);
                        let event = GossipStoreOutEvent::Publish {
                            topic: topic.to_string(),
                            microledger: entry.did.microledger.clone(),
                        };
                        db.events.push_back(event);
                    } else {
                        entry.waiting_publish = true;
                        db.events.push_back(GossipStoreOutEvent::WaitAndPublish(id));
                    }
                }
            }
            GossipStoreInEvent::GossipMutation { last_event_id, did } => {
                let current = db.identities.get_mut(&id);
                match current {
                    None => {
                        let entry = IdEntry::new(did)?;
                        db.identities.insert(id.clone(), entry);
                        db.events
                            .push_back(GossipStoreOutEvent::NotifySubscribers(id));
                    }
                    Some(entry) => {
                        entry.waiting_publish = false;
                        if last_event_id != entry.id_state.last_event_id {
                            entry.did = did.clone();
                            log::info!("Updated id: {:?}", did.id);
                            db.events
                                .push_back(GossipStoreOutEvent::NotifySubscribers(id));
                        } else {
                            log::info!("Skipped id: {:?}", did.id);
                        }
                    }
                }
            }
            GossipStoreInEvent::GossipMessage(msg) => {
                // save message
                db.events.push_back(GossipStoreOutEvent::NotifyId(id));
            }
            GossipStoreInEvent::PairRequest() => {
                
            }
        }

        Ok(())
    }

    fn get_id_peers(&self, id: &[u8]) -> Vec<IdPeer> {
        let db = self.state.lock().unwrap();
        let pair = db.pairs.get(id).unwrap();
        pair.peers.clone()
    }

    fn get_id_subscriber_peers(&self, id: &[u8]) -> Vec<IdPeer> {
        let db = self.state.lock().unwrap();
        let entry = db.identities.get(id).unwrap();
        let mut peers: Vec<IdPeer> = vec![];
        for subscriber in &entry.subscribers{

        }
        peers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {}
}
