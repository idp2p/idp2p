use cid::Cid;
use futures::{channel::mpsc, SinkExt, StreamExt};
use idp2p_common::cbor;
use idp2p_p2p::{
    handler::IdMessageHandler,
    message::{IdGossipMessageKind, IdMessageHandlerRequestKind, IdMessageHandlerResponseKind},
    model::{IdStore, IdVerifier},
};
use libp2p::{
    gossipsub::{self, Behaviour as GossipsubBehaviour, IdentTopic},
    identity::Keypair,
    mdns, noise,
    request_response::{self, cbor::Behaviour as ReqResBehaviour, ProtocolSupport},
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, PeerId, StreamProtocol, Swarm,
};
use serde::{Deserialize, Serialize};
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
    time::Duration,
};

use crate::{app::IdAppEvent, store::InMemoryKvStore};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdRequestKind {
    Meet,
    Message(IdMessageHandlerRequestKind),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdResponseKind {
    MeetResult { username: String, id: String },
    Message(IdMessageHandlerResponseKind),
}

pub(crate) enum IdNetworkCommand {
    SendRequest {
        peer: PeerId,
        req: IdRequestKind,
    },
    Publish {
        topic: IdentTopic,
        payload: IdGossipMessageKind,
    },
}

#[derive(NetworkBehaviour)]
pub(crate) struct Idp2pBehaviour {
    pub(crate) request_response: ReqResBehaviour<IdRequestKind, IdResponseKind>,
    pub(crate) gossipsub: GossipsubBehaviour,
    pub(crate) mdns: mdns::tokio::Behaviour,
}

pub(crate) struct IdNetworkEventLoop<S: IdStore, V: IdVerifier> {
    store: Arc<InMemoryKvStore>,
    swarm: Swarm<Idp2pBehaviour>,
    event_sender: mpsc::Sender<IdAppEvent>,
    cmd_receiver: mpsc::Receiver<IdNetworkCommand>,
    id_handler: IdMessageHandler<S, V>,
}

impl<S: IdStore, V: IdVerifier> IdNetworkEventLoop<S, V> {
    pub fn new(
        port: u16,
        store: Arc<InMemoryKvStore>,
        event_sender: mpsc::Sender<IdAppEvent>,
        cmd_receiver: mpsc::Receiver<IdNetworkCommand>,
        id_handler: IdMessageHandler<S, V>,
    ) -> anyhow::Result<(PeerId, Self)> {
        let swarm = create_swarm(port, )?;
        Ok((
            swarm.local_peer_id().to_owned(),
            Self {
                store,
                swarm,
                event_sender,
                cmd_receiver,
                id_handler,
            },
        ))
    }

    pub(crate) async fn run(mut self) {
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => self.handle_network_event(event).await.unwrap(),
                cmd = self.cmd_receiver.next() => match cmd {
                    Some(cmd) => {
                        let r = self.handle_command(cmd).await;
                        if let Err(e) = r {
                            println!("Error handling command: {}", e);
                        }
                    },
                    None =>  return,
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
                let data = cbor::encode(&payload);
                self.swarm.behaviour_mut().gossipsub.publish(topic, data)?;
            }
        }
        Ok(())
    }

    async fn handle_network_event(
        &mut self,
        event: SwarmEvent<Idp2pBehaviourEvent>,
    ) -> anyhow::Result<()> {
        let mut current_user = self.store.get_current_user().await.unwrap();

        match event {
            SwarmEvent::Behaviour(Idp2pBehaviourEvent::Gossipsub(event)) => match event {
                libp2p::gossipsub::Event::Message {
                    propagation_source: _,
                    message_id: _,
                    message,
                } => {
                    let result = self
                        .id_handler
                        .handle_gossip_message(&message.topic, &message.data)
                        .await?;
                    if let Some(payload) = result {
                        println!("Custom message {:?}", payload);
                    }
                }
                _ => {}
            },
            SwarmEvent::Behaviour(Idp2pBehaviourEvent::RequestResponse(
                request_response::Event::Message { peer, message },
            )) => match message {
                request_response::Message::Request {
                    request, channel, ..
                } => match request {
                    IdRequestKind::Message(req) => {
                        let message = self.id_handler.handle_request_message(peer, req).await?;
                        let message = IdResponseKind::Message(message);
                        self.swarm
                            .behaviour_mut()
                            .request_response
                            .send_response(channel, message)
                            .unwrap();
                    }
                    IdRequestKind::Meet => {
                        let r = self.swarm
                            .behaviour_mut()
                            .request_response
                            .send_response(
                                channel,
                                IdResponseKind::MeetResult {
                                    username: current_user.username.clone(),
                                    id: current_user.id.clone(),
                                },
                            );
                        match r {
                            Ok(_) => {},        
                            Err(e) => {
                                let msg = format!("Failed to send meet result: {e:?}");
                                self.event_sender
                                    .send(IdAppEvent::Other(msg))
                                    .await
                                    .unwrap();
                            }
                        }
                    }
                },
                request_response::Message::Response { response, .. } => match response {
                    IdResponseKind::Message(msg) => {
                        //self.id_handler.handle_response_message(from, message_id, payload)
                        /*self.event_sender
                        .send(IdAppInEvent::GotMessage(msg))
                        .await?;*/
                    }
                    IdResponseKind::MeetResult { username, id } => {
                        current_user.set_other_id(&username, &id, &peer);
                        /*self.swarm
                            .behaviour_mut()
                            .gossipsub
                            .subscribe(&IdentTopic::new(id.as_str()))?;*/
                        let msg = format!("Connected to {} as {}", peer.to_string(), username);

                        self.event_sender
                            .send(IdAppEvent::Other(msg))
                            .await
                            .unwrap();
                    }
                },
            },
            SwarmEvent::Behaviour(Idp2pBehaviourEvent::RequestResponse(request_response::Event::OutboundFailure {
                peer,
                request_id,
                error
            })) => {
                // ---- THIS is where you catch the error for a failed outbound request ----
                eprintln!(
                    "Outbound request to peer {:?} (ID = {:?}) failed: {:?}",
                    peer, request_id, error
                );
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                let msg = format!("Listening on {address}");

                self.event_sender
                    .send(IdAppEvent::Other(msg))
                    .await
                    .unwrap();
            }
            SwarmEvent::Behaviour(Idp2pBehaviourEvent::Mdns(libp2p::mdns::Event::Discovered(
                list,
            ))) => {        
                for (peer_id, multiaddr) in list {
                    if !current_user.peers.contains_key(&peer_id) {
                        current_user.peers.insert(peer_id.clone(), false);
                        self.event_sender
                            .send(IdAppEvent::Other(format!(
                                "Discovered peer {}",
                                peer_id.to_string()
                            )))
                            .await      
                            .unwrap();
                    }
                    self.swarm
                        .behaviour_mut()
                        .gossipsub
                        .add_explicit_peer(&peer_id);
                    self.swarm.add_peer_address(peer_id, multiaddr);

                }
            }
            SwarmEvent::Behaviour(Idp2pBehaviourEvent::Mdns(libp2p::mdns::Event::Expired(
                list,
            ))) => {
                for (peer_id, _multiaddr) in list {
                    self.swarm
                        .behaviour_mut()
                        .gossipsub
                        .remove_explicit_peer(&peer_id);
                }
            }
            _ => {}
        }
        self.store.set_current_user(&current_user).await.unwrap();

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

pub fn create_reqres() -> ReqResBehaviour<IdRequestKind, IdResponseKind> {
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
            let mdns =
                mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?;
            let behaviour = Idp2pBehaviour {
                mdns,
                request_response: create_reqres(),
                gossipsub: create_gossipsub(key)?,
            };
            Ok(behaviour)
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();
    let ip = local_ip_address::local_ip().expect("Could not get local ip");
    swarm.listen_on(format!("/ip4/{ip}/tcp/{port}").parse().expect("Could not parse address"))?;
    Ok(swarm)
}
