use libp2p::{
    swarm::NetworkBehaviour,
    gossipsub::{Behaviour as GossipsubBehaviour, Event as GossipsubEvent},
    mdns::{Event as MdnsEvent, tokio::Behaviour as MdnsBehaviour},
    request_response::{cbor::Behaviour as CborBehaviour, Event as RequestResponseEvent},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NodeRequest {
    KeyEventBlock,
    TimestampBlock{},
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NodeResponse {
    KeyEventBlock(Vec<u8>),
    TimestampBlock(Vec<u8>),
}

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "Idp2pNodeEvent")]
pub struct Idp2pNodeBehaviour {
    pub id_mdns: MdnsBehaviour,
    pub id_gossipsub: GossipsubBehaviour<m>,
    pub id_reqres: CborBehaviour<IdRequest, IdResponse>,
}

#[derive(Debug)]
pub enum Idp2pNodeEvent {
    Mdns(MdnsEvent),
    Gossipsub(GossipsubEvent),
    IdRequestResponse(RequestResponseEvent<IdRequest, IdResponse>),
}

impl From<MdnsEvent> for Idp2pNodeEvent {
    fn from(event: MdnsEvent) -> Self {
        Idp2pNodeEvent::Mdns(event)
    }
}

impl From<GossipsubEvent> for Idp2pNodeEvent {
    fn from(event: GossipsubEvent) -> Self {
        Idp2pNodeEvent::Gossipsub(event)
    }
}

impl From<RequestResponseEvent<IdRequest, IdResponse>> for Idp2pNodeEvent {
    fn from(event: RequestResponseEvent<NodeRequest, NodeResponse>) -> Self {
        Idp2pNodeEvent::NodeRequestResponse(event)
    }
}

impl From<RequestResponseEvent<VerifierRequest, VerifierResponse>> for Idp2pNodeEvent {
    fn from(event: RequestResponseEvent<VerifierRequest, VerifierResponse>) -> Self {
        Idp2pNodeEvent::VerifierRequestResponse(event)
    }
}

impl Idp2pNodeBehaviour {
    pub fn handle_mdns_event(&mut self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(list) => {
                for (peer, _) in list {
                    self.gossipsub.add_explicit_peer(&peer);
                }
            }
            MdnsEvent::Expired(list) => {
                for (peer, _) in list {
                    if !self.mdns.discovered_nodes().any(|p| p == &peer) {
                        self.gossipsub.remove_explicit_peer(&peer);
                    }
                }
            }
        }
    }

    pub fn handle_gossip_event(&mut self, event: GossipsubEvent) {
        match event {
            GossipsubEvent::Message {
                propagation_source,
                message_id,
                message,
            } => if message.topic.as_str().len() == 0 {},
            _ => {}
        }
    }
}
