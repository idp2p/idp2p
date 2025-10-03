use futures::{SinkExt, channel::mpsc, future::FutureExt, select, stream::StreamExt};

use idp2p_common::wasmsg::Wasmsg;
use idp2p_id::types::IdEventReceipt;
use libp2p::{
    Multiaddr, PeerId, StreamProtocol, Swarm,
    gossipsub::{self, Behaviour as GossipsubBehaviour, IdentTopic, TopicHash},
    identity::Keypair,
    noise,
    request_response::{self, ProtocolSupport, cbor::Behaviour as ReqResBehaviour},
    swarm::{NetworkBehaviour, SwarmEvent, dial_opts::DialOpts},
    tcp, yamux,
};
use std::{
    collections::{BTreeSet, HashMap},
    hash::{DefaultHasher, Hash, Hasher},
    sync::{Arc, Mutex},
    time::Duration,
};
use tracing::info;

// pending request to dial
// pending response to deliver

pub trait IdNetworkStore {
    fn set(&self, key: &str, value: &[u8]);
    fn get(&self, key: &str) -> Option<Vec<u8>>;
}

pub enum IdNetworkCommand {
    SendRequest {
        peer_id: PeerId,
        peer_addr: Multiaddr,
        payload: Vec<u8>
    },
    Publish {
        topic: String,
        payload: Vec<u8>,
    },
    Subscribe(IdentTopic),
}

pub enum IdNetworkEvent {}

#[derive(NetworkBehaviour)]
pub struct Idp2pBehaviour {
    pub request_response: ReqResBehaviour<Vec<u8>, bool>,
    pub gossipsub: GossipsubBehaviour,
}

pub struct IdNetworkEventLoop<S: IdNetworkStore> {
    store: Arc<S>,
    swarm: Swarm<Idp2pBehaviour>,
    event_sender: mpsc::Sender<IdNetworkEvent>,
    cmd_receiver: mpsc::Receiver<IdNetworkCommand>,
    pending_requests: HashMap<PeerId, Vec<Vec<u8>>>,
}

impl<S: IdNetworkStore> IdNetworkEventLoop<S> {
    pub fn new(
        port: u16,
        store: S,
        event_sender: mpsc::Sender<IdNetworkEvent>,
        cmd_receiver: mpsc::Receiver<IdNetworkCommand>,
    ) -> anyhow::Result<(PeerId, Self)> {
        let swarm = create_swarm(port)?;
        Ok((
            swarm.local_peer_id().to_owned(),
            Self {
                store: Arc::new(store),
                swarm,
                cmd_receiver,
                event_sender,
                pending_requests: HashMap::new(),
            },
        ))
    }

    pub(crate) async fn run(mut self) {
        loop {
            select! {
                event = self.swarm.next().fuse() => match event {
                    Some(event) => {
                        if let Err(e) = self.handle_network_event(event).await {
                            println!("Error handling network event: {}", e);
                        }
                    }
                    None => return
                },
                cmd = self.cmd_receiver.next().fuse() => match cmd {
                    Some(cmd) => {
                        if let Err(e) = self.handle_command(cmd).await {
                            println!("Error handling command: {}", e);
                        }
                    }
                    None => return,
                }
            }
        }
    }

    async fn handle_command(&mut self, cmd: IdNetworkCommand) -> anyhow::Result<()> {
        use IdNetworkCommand::*;
        match cmd {
            SendRequest {
                peer_id,
                peer_addr,
                payload,
            } => {
                if self.swarm.is_connected(&peer_id) {
                    let _ = self
                        .swarm
                        .behaviour_mut()
                        .request_response
                        .send_request(&peer_id, payload);
                } else {
                    // Queue the request
                    self.pending_requests
                        .entry(peer_id)
                        .or_insert_with(Vec::new)
                        .push(payload);

                    self.swarm.dial(
                        DialOpts::peer_id(peer_id)
                            .addresses(vec![peer_addr])
                            .build(),
                    )?;
                }
            }
            Publish { topic, payload } => {
                info!(
                    "Publishing message: {topic}, payload: {payload:?}",
                    topic = topic.as_str()
                );
                let ident_topic = IdentTopic::new(topic.as_str());
                self.swarm
                    .behaviour_mut()
                    .gossipsub
                    .subscribe(&ident_topic)?;
                let topichash = TopicHash::from_raw(topic.as_str());
                self.swarm
                    .behaviour_mut()
                    .gossipsub
                    .publish(topichash, payload)?;
            }
            Subscribe(topic) => {
                info!(
                    "Subscribing to topic: {topic}",
                    topic = topic.hash().as_str()
                );
                self.swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
            }
        }
        Ok(())
    }

    async fn handle_network_event(
        &mut self,
        event: SwarmEvent<Idp2pBehaviourEvent>,
    ) -> anyhow::Result<()> {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                let msg = format!("Listening on {address}");

                println!("{msg}");
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                println!("Connected to peer: {}", peer_id);

                // Send all pending requests for this peer
                if let Some(pending) = self.pending_requests.remove(&peer_id) {
                    for req in pending {
                        let _ = self
                            .swarm
                            .behaviour_mut()
                            .request_response
                            .send_request(&peer_id, req);
                    }
                }
            }
            SwarmEvent::Behaviour(Idp2pBehaviourEvent::Gossipsub(event)) => match event {
                libp2p::gossipsub::Event::Message {
                    propagation_source: _,
                    message_id: _,
                    message,
                } => {
                    /*
                     if message.is_notify_message() {
                        send a request to provided peer to get actual message
                     }
                    */
                }
                _ => {}
            },
            SwarmEvent::Behaviour(Idp2pBehaviourEvent::RequestResponse(
                request_response::Event::Message { peer, message },
            )) => match message {
                request_response::Message::Request {
                    request, channel, ..
                } => {
                    println!("{:?}", request);
                    // get message content from store
                    // authorize the client
                    // send message
                    self.swarm
                        .behaviour_mut()
                        .request_response
                        .send_response(channel, true)
                        .unwrap()
                }
                request_response::Message::Response { response, .. } => {
                    if response {
                        // handle the message with runtime
                        println!("Message received")
                    }
                }
            },

            other => {
                println!("Swarm event: {other:?}");
            }
        }

        Ok(())
    }
}

pub fn create_gossipsub(key: &Keypair) -> anyhow::Result<GossipsubBehaviour> {
    let message_id_fn = |message: &gossipsub::Message| {
        let mut s = DefaultHasher::new();
        message.data.hash(&mut s);
        gossipsub::MessageId::from(s.finish().to_string())
    };

    // Set a custom gossipsub configuration
    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
        .validation_mode(gossipsub::ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
        .message_id_fn(message_id_fn) // content-address messages. No two messages of the same content will be propagated.
        .build()
        .map_err(|msg| tokio::io::Error::new(tokio::io::ErrorKind::Other, msg))?; // Temporary hack because `build` does not return a proper `std::error::Error`.

    // build a gossipsub network behaviour
    let gossipsub = GossipsubBehaviour::new(
        gossipsub::MessageAuthenticity::Signed(key.clone()),
        gossipsub_config,
    )
    .map_err(anyhow::Error::msg)?;
    Ok(gossipsub)
}

pub fn create_reqres() -> ReqResBehaviour<Vec<u8>, bool> {
    libp2p::request_response::cbor::Behaviour::new(
        [(StreamProtocol::new("/idp2p/1"), ProtocolSupport::Full)],
        libp2p::request_response::Config::default(),
    )
}

pub fn create_swarm(port: u16) -> anyhow::Result<Swarm<Idp2pBehaviour>> {
    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_behaviour(|key| {
            let behaviour = Idp2pBehaviour {
                request_response: create_reqres(),
                gossipsub: create_gossipsub(key)?,
            };
            Ok(behaviour)
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();
    let ip = local_ip_address::local_ip().expect("Could not get local ip");
    swarm.listen_on(
        format!("/ip4/{ip}/tcp/{port}")
            .parse()
            .expect("Could not parse address"),
    )?;
    Ok(swarm)
}
