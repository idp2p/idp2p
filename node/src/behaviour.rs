use core::{
    idp2p_proto::{IdGossipMessageWithTopic, IdGossipRequest},
    protocol::codec::{IdCodec, IdRequest, IdResponse},
};
use std::collections::HashMap;

use core::prost::Message;
use libp2p::{
    gossipsub::{Gossipsub, GossipsubEvent},
    mdns::{Mdns, MdnsEvent},
    request_response::{RequestResponse, RequestResponseEvent, RequestResponseMessage},
    NetworkBehaviour, PeerId,
};

use crate::error::GossipError;

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "IdentityEvent")]
pub struct IdentityBehaviour {
    pub mdns: Mdns,
    pub request_response: RequestResponse<IdCodec>,
    pub gossipsub: Gossipsub,
    #[behaviour(ignore)]
    identities: HashMap<Vec<u8>, IdEntry>
}

#[derive(Debug)]
pub enum IdentityGossipEvent {
    Mdns(MdnsEvent),
    Gossipsub(GossipsubEvent),
    RequestResponse(RequestResponseEvent<IdRequest, IdResponse>),
}

impl From<MdnsEvent> for IdentityGossipEvent {
    fn from(event: MdnsEvent) -> Self {
        IdentityGossipEvent::Mdns(event)
    }
}

impl From<GossipsubEvent> for IdentityGossipEvent {
    fn from(event: GossipsubEvent) -> Self {
        Self::Gossipsub(event)
    }
}

impl From<RequestResponseEvent<IdRequest, IdResponse>> for IdentityGossipEvent {
    fn from(event: RequestResponseEvent<IdRequest, IdResponse>) -> Self {
        Self::RequestResponse(event)
    }
}
type ReqResEvent = RequestResponseEvent<IdRequest, IdResponse>;
impl IdentityGossipBehaviour {
    pub fn handle_mdns_event(&mut self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(list) => {
                for (peer, _) in list {
                    self.gossipsub.add_explicit_peer(&peer);
                }
            }
            MdnsEvent::Expired(list) => {
                for (peer, _) in list {
                    if !self.mdns.has_node(&peer) {
                        self.gossipsub.remove_explicit_peer(&peer);
                    }
                }
            }
        }
    }

    pub fn handle_request(&mut self, event: ReqResEvent) -> Result<(), GossipError> {
        if let RequestResponseEvent::Message { message, .. } = event {
            match message {
                RequestResponseMessage::Request {
                    request, channel, ..
                } => {
                    let req_result = IdGossipRequest::decode(&*request.0);
                    match req_result {
                        Ok(_) => todo!(),
                        Err(_) => todo!(),
                    };
                    /*self.request_response
                    .send_response(channel, IdNodeResponse("Hoyy".to_owned()))
                    .expect("Connection to peer sto be still open.");*/
                }
                RequestResponseMessage::Response {
                    request_id: _,
                    response:_,
                } => {},
            }
        }
        Ok(())
    }

    pub fn handle_gossip_event(&self, event: GossipsubEvent) -> Result<(), GossipError> {
        if let GossipsubEvent::Message {
            propagation_source: _,
            message_id: _,
            message,
        } = event
        {
            let topic = message.topic.to_string();
            if let Some(store) = self.topics.get(&topic) {
                let msg = IdGossipMessageWithTopic::decode(&*message.data)?;
                for peer in store {
                    /*let req = IdNodeRequest{
                        message_type: NodeMessageType::Notify(),
                    }*/
                }
            }
        }
        Ok(())
    }
}
