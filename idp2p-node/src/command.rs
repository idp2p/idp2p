use crate::message::{IdentityMessage, IdentityMessagePayload};
use crate::swarm::IdentityGossipBehaviour;
use idp2p_common::store::IdStore;
use idp2p_core::did::Identity;
use libp2p::gossipsub::IdentTopic;

#[derive(PartialEq, Debug, Clone)]
pub enum IdentityCommand {
    Get { id: String },
    Post { did: Identity },
    SendJwm { jwm: String },
}

impl IdentityCommand {
    fn handle(&self, behaviour: &mut IdentityGossipBehaviour, store: impl IdStore) {
        match self {
            Self::Get { id } => handle_get(id, behaviour),
            Self::Post { did } => println!(""),
            Self::SendJwm { jwm } => println!(""),
        }
    }
}

pub fn handle_get(id: &String, behaviour: &mut IdentityGossipBehaviour){
    let gossipsub_topic = IdentTopic::new(id.clone());
    behaviour.gossipsub.subscribe(&gossipsub_topic).unwrap();
    let mes = IdentityMessage::new(IdentityMessagePayload::Get);
    behaviour.publish(id.clone(), mes);
}