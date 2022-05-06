use std::collections::{HashMap, VecDeque};
use std::io::Cursor;
/*use idp2p_core::did::microledger::MicroLedgerInception;
use libp2p::{
    core::{connection::ConnectionId, ConnectedPoint, Endpoint},
    request_response::RequestResponse,
    swarm::{NetworkBehaviour, OneShotHandler},
    Multiaddr, PeerId,
};
use prost::Message;
//use message_proto::{idp2p_message::Message, PublishMessage};
use protocol::IdCodec;
mod message_proto {
    include!(concat!(env!("OUT_DIR"), "/idp2p.pb.rs"));
}
pub mod protocol;
pub enum IdGossipEvent {
    Published,
    Subscribed,
    Received,
}

pub struct IdentityGossipBehaviour {
    events: VecDeque<IdGossipEvent>,
    // Id - Client
    registrations: HashMap<String, String>, //
}

impl NetworkBehaviour for IdentityGossipBehaviour {
    //type ConnectionHandler;

    type OutEvent = IdGossipEvent;

    fn new_handler(&mut self) -> Self::ConnectionHandler {
        Default::default()
    }

    fn inject_event(
        &mut self,
        peer_id: PeerId,
        connection: ConnectionId,
        event: <<Self::ConnectionHandler as libp2p::swarm::IntoConnectionHandler>::Handler as libp2p::swarm::ConnectionHandler>::OutEvent,
    ) {
        todo!()
    }

    fn poll(
        &mut self,
        cx: &mut std::task::Context<'_>,
        params: &mut impl libp2p::swarm::PollParameters,
    ) -> std::task::Poll<libp2p::swarm::NetworkBehaviourAction<Self::OutEvent, Self::ConnectionHandler>> {
        todo!()
    }
}*/

/*fn xyz() {
    let msg = message_proto::Idp2pMessage {
        from: vec![],
        seqno: vec![],
        message: Some(Message::Publish(PublishMessage {
            peer_id: vec![],
            jwm: vec![],
        })),
    };
    match msg.message.unwrap() {
        Message::Subscribe(m) => {}
        _ => {}
    }
    let microledger = message_proto::Microledger{
        incseption: inception,

    };
}*/
/*#[derive(Clone, PartialEq, Message)]
struct IdP2pMessage {
    #[prost(bytes, tag = "1")]
    pub from: Vec<u8>,
    #[prost(bytes, tag = "2")]
    pub seqno: Vec<u8>,
    #[prost(oneof = "Idp2pMessageType", tags = "3")]
    pub payload: Option<Idp2pMessageType>,
}

#[derive(Eq, PartialOrd, Ord, Clone, PartialEq, ::prost::Oneof)]
pub enum Idp2pMessageType {
    #[prost(string, tag = "1")]
    Publish(::prost::alloc::string::String),
    #[prost(message, tag = "2")]
    Subscribe(::prost::alloc::string::String),
    #[prost(string, tag = "3")]
    Connect(::prost::alloc::string::String),
}*/
