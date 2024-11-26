use anyhow::Result;
use futures::{channel::mpsc::*, StreamExt};
use libp2p::{gossipsub::{IdentTopic, Topic}, request_response, swarm::SwarmEvent, Swarm};
use std::{
    collections::HashMap,
    error::Error,
    sync::{Arc, Mutex},
};
use wasmtime::{component::Component, Engine};

use crate::{
    behaviour::{create_swarm, Idp2pBehaviour, Idp2pBehaviourEvent},
    utils,
};

pub struct InMemoryKvStore {
    pub state: Mutex<HashMap<String, Vec<u8>>>,
}

impl InMemoryKvStore {
    fn new() -> Self {
        Self {
            state: Mutex::new(HashMap::new()),
        }
    }

    fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let state = self.state.lock().unwrap();
        if let Some(value) = state.get(key) {
            return Ok(Some(value.to_vec()));
        }
        Ok(None)
    }

    fn put(&self, key: &str, value: &[u8]) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.insert(key.to_owned(), value.to_vec());
        Ok(())
    }
}

pub struct IdMessageHandler {
    engine: Engine,
    kv_store: Arc<InMemoryKvStore>,
    swarm: Swarm<Idp2pBehaviour>,
    id_components: HashMap<String, Component>,
    p2p_components: HashMap<String, Component>,
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
        self.swarm.behaviour_mut().gossipsub.publish(IdentTopic::new(id), b"data");
    }

    async fn handle_event(&mut self, event: SwarmEvent<Idp2pBehaviourEvent>) {
        match event {
            SwarmEvent::Behaviour(Idp2pBehaviourEvent::Mdns(libp2p::mdns::Event::Discovered(
                list,
            ))) => {
                for (peer_id, _multiaddr) in list {
                    println!("mDNS discovered a new peer: {peer_id}");
                    self.swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
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
                //handler.handle_gossip_message(event).await?;
            }
            SwarmEvent::Behaviour(Idp2pBehaviourEvent::RequestResponse(request_response::Event::Message { message, .. })) => {
                match message {
                    request_response::Message::Request {
                        request, channel, ..
                    } => {
                        
                    }
                    request_response::Message::Response {
                        request_id,
                        response,
                    } => {
                        
                    }
                }
                todo!()
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Local node is listening on {address}");
            }
            _ => {}
        }
    }

    pub(crate) async fn run(mut self) {
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => self.handle_event(event).await,
                
            }
        }
    }
}
