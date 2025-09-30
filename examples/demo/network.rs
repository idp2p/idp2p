use futures::{channel::mpsc, future::FutureExt, select, stream::StreamExt, SinkExt};

use idp2p_common::wasmsg::Wasmsg;
use libp2p::{
    gossipsub::{self, Behaviour as GossipsubBehaviour, IdentTopic, TopicHash},
    identity::Keypair,
    noise,
    request_response::{self, json::Behaviour as ReqResBehaviour, ProtocolSupport},
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, PeerId, StreamProtocol, Swarm,
};
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
    time::Duration,
};
use tracing::info;

pub(crate) enum IdNetworkCommand {
    SendRequest {
        peer: PeerId,
        req: Wasmsg,
    },
    Publish {
        topic: String,
        payload: Vec<u8>,
    },
    Subscribe(IdentTopic),
}

#[derive(NetworkBehaviour)]
pub(crate) struct Idp2pBehaviour {
    pub(crate) request_response: ReqResBehaviour<Wasmsg, bool>,
    pub(crate) gossipsub: GossipsubBehaviour,
}

pub(crate) struct IdNetworkEventLoop {
    swarm: Swarm<Idp2pBehaviour>,
    //event_sender: mpsc::Sender<IdAppEvent>,
    cmd_receiver: mpsc::Receiver<IdNetworkCommand>,
    //id_handler: IdMessageHandler<S, V>,
}

impl IdNetworkEventLoop {
    pub fn new(
        port: u16,
        cmd_receiver: mpsc::Receiver<IdNetworkCommand>,
    ) -> anyhow::Result<(PeerId, Self)> {
        let swarm = create_swarm(port)?;
        Ok((
            swarm.local_peer_id().to_owned(),
            Self {
                swarm,
                cmd_receiver,
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
            SendRequest { peer, req } => {
                let _ = self
                    .swarm
                    .behaviour_mut()
                    .request_response
                    .send_request(&peer, req);
            }
            Publish { topic, payload } => {
                info!(
                    "Publishing message: {topic}, payload: {payload:?}",
                    topic = topic.as_str()
                );
                /*let mesh_peers = self.swarm.behaviour_mut().gossipsub.mesh_peers(&topic);
                for mpeer in mesh_peers {
                    info!("Mesh peer: {}", mpeer);
                }*/

                let ident_topic = IdentTopic::new(topic.as_str());
                self.swarm
                    .behaviour_mut()
                    .gossipsub
                    .subscribe(&ident_topic)?;
                let data = idp2p_common::cbor::encode(&payload);
                let topichash = TopicHash::from_raw(topic.as_str());
                self.swarm.behaviour_mut().gossipsub.publish(topichash, data)?;
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
            SwarmEvent::Behaviour(Idp2pBehaviourEvent::Gossipsub(event)) => match event {
                libp2p::gossipsub::Event::Message {
                    propagation_source: _,
                    message_id: _,
                    message,
                } => {
                    
                }
                _ => {}
            },
            SwarmEvent::Behaviour(Idp2pBehaviourEvent::RequestResponse(
                request_response::Event::Message { peer, message },
            )) => match message {
                request_response::Message::Request {
                    request, channel, ..
                } => {
                    // fire handle wasmsg event
                    println!("{:?}", request);
                    self.swarm
                            .behaviour_mut()
                            .request_response
                            .send_response(channel, true)
                            .unwrap()
                },
                request_response::Message::Response { response, .. } => {
                    if response {
                        println!("Message received")
                    }
                },
            },
            SwarmEvent::Behaviour(Idp2pBehaviourEvent::RequestResponse(
                request_response::Event::OutboundFailure {
                    peer,
                    request_id,
                    error,
                },
            )) => {
                eprintln!(
                    "Outbound request to peer {:?} (ID = {:?}) failed: {:?}",
                    peer, request_id, error
                );
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                let msg = format!("Listening on {address}");

                println!("{msg}");
            }
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

pub fn create_reqres() -> ReqResBehaviour<Wasmsg, bool> {
    libp2p::request_response::json::Behaviour::new(
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
