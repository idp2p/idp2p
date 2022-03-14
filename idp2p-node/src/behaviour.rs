use idp2p_common::anyhow::Result;
use idp2p_common::serde_json;
use idp2p_core::message::{IdentityMessage, IdentityMessagePayload};
use idp2p_core::store::IdStore;
use libp2p::relay::v2::relay::{self, Relay};
use libp2p::{
    gossipsub::{Gossipsub, GossipsubEvent, IdentTopic},
    identify::{Identify, IdentifyEvent},
    rendezvous,
    swarm::NetworkBehaviourEventProcess,
    NetworkBehaviour,
};

#[derive(NetworkBehaviour)]
#[behaviour(event_process = true)]
pub struct IdentityGossipBehaviour {
    pub identify: Identify,
    pub rendezvous: rendezvous::server::Behaviour,
    pub gossipsub: Gossipsub,
    pub relay: Relay,
    #[behaviour(ignore)]
    pub id_store: IdStore,
}

impl IdentityGossipBehaviour {
    pub fn publish(&mut self, id: &str, mes: IdentityMessage) {
        let gossip_topic = IdentTopic::new(id);
        let json_str = serde_json::to_string(&mes).unwrap();
        let result = self.gossipsub.publish(gossip_topic, json_str.as_bytes());
        match result {
            Ok(_) => println!("Published id: {}", id),
            Err(e) => println!("Publish error, {:?}", e),
        }
    }

    pub fn subscribe(&mut self, id: &str) -> Result<()> {
        let gossip_topic = IdentTopic::new(id);
        self.gossipsub.subscribe(&gossip_topic)?;
        Ok(())
    }
}

impl NetworkBehaviourEventProcess<GossipsubEvent> for IdentityGossipBehaviour {
    fn inject_event(&mut self, message: GossipsubEvent) {
        println!("Got message: {:?}", message);
        if let GossipsubEvent::Message {
            propagation_source: _,
            message_id: _,
            message,
        } = message
        {
            let topic = message.topic.to_string();
            let err_msg = "Message is not well-formed. It should be json";
            let message: IdentityMessage = serde_json::from_slice(&message.data).expect(err_msg);
            match &message.payload {
                IdentityMessagePayload::Get => {
                    self.id_store.handle_get(&topic);
                }
                IdentityMessagePayload::Post { digest, identity } => {
                    let result = self.id_store.handle_post(digest, identity);
                    match result {
                        Ok(_) => {}
                        Err(_) => {}
                    }
                }
                _ => {}
            }
        }
    }
}

impl NetworkBehaviourEventProcess<IdentifyEvent> for IdentityGossipBehaviour {
    fn inject_event(&mut self, event: IdentifyEvent) {
        if let IdentifyEvent::Received { peer_id, .. } = event {
            println!("Identify event: {}", peer_id);
        }
    }
}

impl NetworkBehaviourEventProcess<relay::Event> for IdentityGossipBehaviour {
    fn inject_event(&mut self, event: relay::Event) {
        println!("{:?}", event);
    }
}

impl NetworkBehaviourEventProcess<rendezvous::server::Event> for IdentityGossipBehaviour {
    fn inject_event(&mut self, e: rendezvous::server::Event) {
        println!("{:?}", e);
    }
}
