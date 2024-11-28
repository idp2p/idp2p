use anyhow::Result;
use futures::{channel::mpsc::*, StreamExt};
use idp2p_common::content::Content;
use libp2p::{
    gossipsub::{IdentTopic, Topic},
    request_response,
    swarm::SwarmEvent,
    Swarm,
};
use store::InMemoryKvStore;
use std::{
    collections::HashMap,
    error::Error,
    sync::{Arc, Mutex},
};
use wasmtime::{component::Component, Engine};

use crate::{
    network::{create_swarm, Idp2pBehaviour, Idp2pBehaviourEvent},
    utils,
};

mod store;
mod wasm;

pub struct IdMessageHandler {
    store: Arc<InMemoryKvStore>,
    swarm: Swarm<Idp2pBehaviour>,
}

impl IdMessageHandler {
    pub fn new(port: u16) -> Result<Self, Box<dyn Error>> {
        let mut swarm: Swarm<Idp2pBehaviour> = create_swarm(port)?;
        //let (sender, receiver) = channel(0);
        let id = utils::generate_id(swarm.local_peer_id())?;
        println!("Id {}", id.id.to_string());
        let kv = Arc::new(InMemoryKvStore::new());
        //let mut handler = IdMessageHandler::new(kv.clone(), sender)?;

        todo!()
    }

    pub fn resolve(&mut self, id: &str) {
        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(IdentTopic::new(id), b"data").unwrap();
    }

    async fn handle_event(&mut self, event: SwarmEvent<Idp2pBehaviourEvent>) -> anyhow::Result<()> {
        match event {
            SwarmEvent::Behaviour(Idp2pBehaviourEvent::Mdns(libp2p::mdns::Event::Discovered(
                list,
            ))) => {
                for (peer_id, _multiaddr) in list {
                    println!("mDNS discovered a new peer: {peer_id}");
                    self.swarm
                        .behaviour_mut()
                        .gossipsub
                        .add_explicit_peer(&peer_id);
                }
            }
            SwarmEvent::Behaviour(Idp2pBehaviourEvent::Mdns(libp2p::mdns::Event::Expired(
                list,
            ))) => {
                for (peer_id, _multiaddr) in list {
                    println!("mDNS discover peer has expired: {peer_id}");
                    self.swarm
                        .behaviour_mut()
                        .gossipsub
                        .remove_explicit_peer(&peer_id);
                }
            }
            SwarmEvent::Behaviour(Idp2pBehaviourEvent::Gossipsub(event)) => {
                match event {
                    libp2p::gossipsub::Event::Message {
                        propagation_source: _,
                        message_id: _,
                        message,
                    } => {
                        let content = Content::from_bytes(message.data.as_slice())?;
                        //handler.handle_gossip_message(event).await?;
                    }
                    _=> {}
                }
     
            }
            SwarmEvent::Behaviour(Idp2pBehaviourEvent::RequestResponse(
                request_response::Event::Message { message, .. },
            )) => {
                match message {
                    request_response::Message::Request {
                        request, channel, ..
                    } => {}
                    request_response::Message::Response {
                        request_id,
                        response,
                    } => {}
                }
                todo!()
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Local node is listening on {address}");
            }
            _ => {}
        }
        Ok(())
    }

    pub(crate) async fn run(mut self) -> anyhow::Result<()>{
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => self.handle_event(event).await?,

            }
        }
    }
}
