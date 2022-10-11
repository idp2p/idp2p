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
pub enum GossipStoreOutEvent {
    Publish(PairsubMessage),
}

#[derive(PartialEq, Debug, Clone)]
pub struct IdEntry {
    // Identity ledger
    did: Identity,
    // Current state
    id_state: IdentityState,
}

struct State {
    auth_keypair: Idp2pAuthenticationKeypair,
    agree_key: Idp2pAgreementKeypair,
    identities: HashMap<Id, IdEntry>,
    events: VecDeque<GossipStoreOutEvent>,
    resolve_reqs: HashMap<String, Id>,
}

pub struct IdStore {
    state: Mutex<GossipState>,
}

impl IdEntry {
    pub fn new(did: Identity) -> Result<Self, Idp2pGossipError> {
        let id_state = did.verify(None)?;
        Ok(Self {
            did: did,
            id_state: id_state,
        })
    }
}

impl IdStore {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(GossipState {
                identities: HashMap::new(),
                events: VecDeque::new(),
            }),
        }
    }

    pub fn handle_pairsub_message(&self, msg: PairsubMessage){
        let mut db = self.state.lock().unwrap();
        
        // if topic is not id owner and resolve_reqs has topic  
        // else if  
    }

    fn handle_resolve_request(&self){
        let mut db = self.state.lock().unwrap();
        let id = Idp2pBase::decode(topic)?;
        let entry = db.identities.get_mut(&id);
        if let Some(entry) = entry {
            log::info!("Published id: {:?}", topic);
            let event = GossipStoreOutEvent::Publish {
                topic: topic.to_string(),
                microledger: entry.did.microledger.clone(),
            };
            db.events.push_back(event);
        }
    }
    fn handle_resolve_response(&self){
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

    fn handle_oneshot_message(&self, ){
        
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {}
}
