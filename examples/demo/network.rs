use cid::Cid;
use futures::{channel::mpsc, SinkExt, StreamExt};
use idp2p_common::cbor;
use idp2p_p2p::{
    handler::{IdHandlerGossipCommand, IdMessageHandler},
    message::IdGossipMessageKind,
    store::KvStore,
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
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use crate::{
    app::{IdAppInEvent, IdAppOutEvent},
    IdUser,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdRequestKind {
    Meet,
    Message(Cid),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdResponseKind {
    MeetResult { username: String, id: Cid },
    Message(String),
}

#[derive(NetworkBehaviour)]
pub(crate) struct Idp2pBehaviour {
    pub(crate) request_response: ReqResBehaviour<IdRequestKind, IdResponseKind>,
    pub(crate) gossipsub: GossipsubBehaviour,
    pub(crate) mdns: mdns::tokio::Behaviour,
}

pub(crate) struct IdNetworkEventLoop<S: KvStore> {
    current_user: String,
    store: Arc<S>,
    swarm: Swarm<Idp2pBehaviour>,
    event_sender: mpsc::Sender<IdAppInEvent>,
    event_receiver: mpsc::Receiver<IdAppOutEvent>,
    id_handler: Arc<IdMessageHandler<S>>,
}

impl<S: KvStore> IdNetworkEventLoop<S> {
    pub fn new(
        current_user: String,
        store: Arc<S>,
        event_sender: mpsc::Sender<IdAppInEvent>,
        event_receiver: mpsc::Receiver<IdAppOutEvent>,
        id_handler: Arc<IdMessageHandler<S>>,
    ) -> anyhow::Result<(PeerId, Self)> {
        let port = match current_user.as_str() {
            "alice" => 43727,
            "bob" => 43728,
            "dog" => 43729,
            _ => panic!("Unknown user"),
        };
        let swarm = create_swarm(port)?;
        Ok((
            swarm.local_peer_id().to_owned(),
            Self {
                current_user,
                store,
                swarm,
                event_sender,
                event_receiver,
                id_handler,
            },
        ))
    }

    pub fn get_user(&self, username: &str) -> IdUser {
        let user = self
            .store
            .get(&format!("/users/{}", username))
            .unwrap()
            .unwrap();
        cbor::decode(&user).unwrap()
    }

    pub(crate) async fn run(mut self) {
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => self.handle_network_event(event).await.unwrap(),
                app_event = self.event_receiver.next() => match app_event {
                    Some(app_event) => self.handle_app_event(app_event).await.unwrap(),
                    None =>  return,
                },
            }
        }
    }

    async fn handle_app_event(&mut self, event: IdAppOutEvent) -> anyhow::Result<()> {
        use IdAppOutEvent::*;
        match event {
            SendMessage(message) => {
                println!("Sending message: {}", message);
            }
            Connect(cid) => {
                //IdGossipMessageKind::Resolve;
                //self.swarm.behaviour_mut().gossipsub.publish(topic, data)
            },
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
                    let payload: IdGossipMessageKind = cbor::decode(&message.data)?;
                    let cmd = self
                        .id_handler
                        .handle_gossip_message(&message.topic, &payload)
                        .await?;
                    use IdHandlerGossipCommand::*;
                    match cmd {
                        Publish { topic, payload } => {
                            self.swarm
                                .behaviour_mut()
                                .gossipsub
                                .publish(topic, payload)
                                .unwrap();
                        }
                        Request { peer, message_id } => {
                            let req = IdRequestKind::Message(Cid::from_str(&message_id).unwrap());
                            self.swarm
                                .behaviour_mut()
                                .request_response
                                .send_request(&peer, req);
                        }
                        None => {}
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
                    IdRequestKind::Message(cid) => {
                        let message = self.id_handler.handle_request_message(peer, cid).await?;
                        let message = cbor::decode(&message)?;
                        self.swarm
                            .behaviour_mut()
                            .request_response
                            .send_response(channel, message)
                            .unwrap();
                    }
                    IdRequestKind::Meet => {
                        let user_id = self.get_user(&self.current_user).id.unwrap().clone();
                        self.swarm
                            .behaviour_mut()
                            .request_response
                            .send_response(
                                channel,
                                IdResponseKind::MeetResult {
                                    username: self.current_user.clone(),
                                    id: user_id,
                                },
                            )
                            .unwrap();
                    }
                },
                request_response::Message::Response { response, .. } => match response {
                    IdResponseKind::Message(msg) => {
                        self.event_sender
                            .send(IdAppInEvent::GotMessage(msg))
                            .await?;
                    }
                    IdResponseKind::MeetResult { username, id } => {
                        let user = self
                            .store
                            .get(&format!("/users/{}", username))
                            .unwrap()
                            .unwrap();
                        let mut user = IdUser::from_bytes(&user);
                        user.id = Some(id);
                        self.store
                            .put(&format!("/users/{}", username), &user.to_bytes())
                            .unwrap();
                        self.swarm
                            .behaviour_mut()
                            .gossipsub
                            .subscribe(&IdentTopic::new(id.to_string()))?;
                    }
                },
            },
            SwarmEvent::NewListenAddr { address, .. } => {
                self.event_sender
                    .send(IdAppInEvent::ListenOn(address.to_string()))
                    .await
                    .unwrap();
            }
            SwarmEvent::Behaviour(Idp2pBehaviourEvent::Mdns(libp2p::mdns::Event::Discovered(
                list,
            ))) => {
                for (peer_id, _multiaddr) in list {
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
                    self.swarm
                        .behaviour_mut()
                        .gossipsub
                        .remove_explicit_peer(&peer_id);
                }
            }

            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                let _ = self
                    .swarm
                    .behaviour_mut()
                    .request_response
                    .send_request(&peer_id, IdRequestKind::Meet);
            }
            _ => {}
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

    swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{port}").parse().unwrap())?;
    Ok(swarm)
}
