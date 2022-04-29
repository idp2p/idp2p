use std::collections::{HashMap, VecDeque};
use std::io::Cursor;
use idp2p_core::did::microledger::MicroLedgerInception;
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

/*impl NetworkBehaviour for IdentityGossipBehaviour {
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

#[derive(Clone, PartialEq)]
pub struct Idp2pPublicKey {
    pub r#type: i32,
    pub public: Vec<u8>,
}

#[derive(Clone, PartialEq)]
pub struct Idp2pPublicKeyDigest {
    pub r#type: i32,
    pub digest: Vec<u8>,
}

#[derive(Clone, PartialEq)]
pub struct Idp2pAgreementPublicKey {
    pub r#type: i32,
    pub public: Vec<u8>,
}

#[derive(Clone, PartialEq)]
pub struct MicroledgerInception {
    pub next_key: Idp2pPublicKeyDigest,
    pub recovery_key: Idp2pPublicKeyDigest,
    pub changes: Vec<MicroledgerChange>,
}

#[derive(Clone, PartialEq)]
pub struct Microledger {
    pub inception: MicroLedgerInception,
    pub events: Vec<EventLog>,
}

#[derive(Clone, PartialEq)]
pub struct EventLog {
    pub payload: EventLogPayload,
    pub proof: Vec<u8>,
}

#[derive(Clone, PartialEq)]
pub struct EventLogPayload {
    pub previous: Vec<u8>,
    pub signer_key: Vec<u8>,
    pub next_key: Idp2pPublicKeyDigest,
    pub timestamp: i64,
    pub changes: Vec<MicroledgerChange>,
}

impl EventLogPayload {
    fn to_bytes(&self) {
        let m_p = message_proto::EventLogPayload {
            next_key_type: self.next_key.r#type,
            next_key_digest: self.next_key.digest.clone(),
            timestamp: self.timestamp,
            previous: self.previous.clone(),
            signer_key: self.signer_key.clone(),
            changes: vec![]//self.changes.clone().into_iter().map(|x| x.into()).collect(),
        };
        m_p.encode_to_vec();
        let buf: Vec<u8> = vec![];
        let m = message_proto::MicroledgerChange::decode(&mut Cursor::new(&buf)).unwrap();
    }
}

#[derive(Clone, PartialEq)]
pub enum MicroledgerChange {
    SetProof { key: Vec<u8>, value: Vec<u8> },
    SetAssertionKey(Idp2pPublicKey),
    SetAuthenticationKey(Idp2pPublicKey),
    SetAgreementKey(Idp2pAgreementPublicKey),
}

impl Into<message_proto::MicroledgerChange> for  MicroledgerChange {
    fn into(self) -> message_proto::MicroledgerChange {
        todo!()
    }
    /*fn into(change: MicroledgerChange) -> Self {
        if let Some(change) = change{
            let ch
            = match change{
               message_proto::MicroledgerChange::SetProof{} => MicroledgerChange::SetProof{};
            }
            MicroledgerChange::SetProof({key: vec![], value: vec![]})
        }
        MicroledgerChange::SetProof {
            key: vec![],
            value: vec![],
        }
    }*/
}

#[derive(Clone, PartialEq)]
pub struct SetProof {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

pub enum Idp2pKeyType {
    Idp2pEd25519Key = 0,
}
