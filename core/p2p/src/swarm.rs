use std::borrow::BorrowMut;

use libp2p::{gossipsub, mdns, request_response, swarm::NetworkBehaviour};

use crate::model::{MessageRequest, MessageResponse};

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "Idp2pEvent")]
pub struct Idp2pBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub reqres: request_response::cbor::Behaviour<MessageRequest, MessageResponse>,
}

#[derive(Debug)]
pub enum Idp2pEvent {
    Mdns(mdns::Event),
    Gossipsub(gossipsub::Event),
    RequestResponse(request_response::Event<MessageRequest, MessageResponse>),
}

impl From<mdns::Event> for Idp2pEvent {
    fn from(event: mdns::Event) -> Self {
        Idp2pEvent::Mdns(event)
    }
}

impl From<gossipsub::Event> for Idp2pEvent {
    fn from(event: gossipsub::Event) -> Self {
        Idp2pEvent::Gossipsub(event)
    }
}

impl From<request_response::Event<MessageRequest, MessageResponse>> for Idp2pEvent {
    fn from(event: request_response::Event<MessageRequest, MessageResponse>) -> Self {
        Idp2pEvent::RequestResponse(event)
    }
}

impl Idp2pBehaviour {
    pub fn handle_mdns_event(&mut self, event: mdns::Event) {
        match event {
            mdns::Event::Discovered(list) => {
                for (peer, _) in list {
                    self.gossipsub.add_explicit_peer(&peer);
                }
            }
            mdns::Event::Expired(list) => {
                for (peer, _) in list {
                    if !self.mdns.has_node(&peer) {
                        self.gossipsub.remove_explicit_peer(&peer);
                    }
                }
            }
        }
    }

    pub fn handle_gossip_event(&mut self, event: gossipsub::Event) {
        match event {
            gossipsub::Event::Message {
                propagation_source,
                message_id,
                message,
            } => {
                if message.topic.as_str().len() == 0 {
                    
                }
            },
            _ => {}
        }
    }
}
