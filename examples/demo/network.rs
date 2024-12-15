use futures::{channel::mpsc, SinkExt, StreamExt};
use idp2p_common::cbor;
use idp2p_p2p::{
    handler::{IdHandlerInboundEvent, IdHandlerOutboundEvent}, message::IdGossipMessageKind, store::KvStore
};
use libp2p::{
    gossipsub::{self, Behaviour as GossipsubBehaviour},
    identity::Keypair,
    mdns, noise,
    request_response::{self, cbor::Behaviour as ReqResBehaviour, ProtocolSupport},
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, StreamProtocol, Swarm,
};
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
    time::Duration,
};

use crate::app::IdAppEvent;

#[derive(NetworkBehaviour)]
pub(crate) struct Idp2pBehaviour {
    pub(crate) request_response: ReqResBehaviour<Vec<u8>, ()>,
    pub(crate) gossipsub: GossipsubBehaviour,
    pub(crate) mdns: mdns::tokio::Behaviour,
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

pub fn create_reqres() -> ReqResBehaviour<Vec<u8>, ()> {
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

    swarm.listen_on(format!("/ip4/127.0.0.1/tcp/{port}").parse().unwrap())?;
    Ok(swarm)
}

pub(crate) struct IdNetworkEventLoop {
    swarm: Swarm<Idp2pBehaviour>,
    app_event_sender: mpsc::Sender<IdAppEvent>,
    event_sender: mpsc::Sender<IdHandlerInboundEvent>,
    event_receiver: mpsc::Receiver<IdHandlerOutboundEvent>,
}

impl IdNetworkEventLoop {
    pub fn new(
        port: u16,
        app_event_sender: mpsc::Sender<IdAppEvent>,
        event_sender: mpsc::Sender<IdHandlerInboundEvent>,
        event_receiver: mpsc::Receiver<IdHandlerOutboundEvent>,
    ) -> anyhow::Result<Self> {
        let swarm = create_swarm(port)?;
        Ok(Self {
            swarm,
            app_event_sender,
            event_sender,
            event_receiver,
        })
    }

    pub(crate) async fn run(mut self) {
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => self.handle_network_event(event).await.unwrap(),
                out_event = self.event_receiver.next() => match out_event {
                    Some(out_event) => self.handle_out_event(out_event).await.unwrap(),
                    None =>  return,
                },
            }
        }
    }

    async fn handle_out_event(&mut self, event: IdHandlerOutboundEvent) -> anyhow::Result<()> {
        use IdHandlerOutboundEvent::*;
        match event {
            RequiredPublish { topic, payload } => {
                self.swarm
                    .behaviour_mut()
                    .gossipsub
                    .publish(topic, payload)
                    .unwrap();
            },
            RequiredRequest { peer, message_id } => {
                self.swarm
                    .behaviour_mut()
                    .request_response
                    .send_request(&peer, message_id.as_bytes().to_vec());
            },
            RequiredResponse {
                message_id,
                payload,
            } => {
                /*self.swarm
                    .behaviour_mut()
                    .request_response
                    .send_response();*/
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
                    self.event_sender
                        .send(IdHandlerInboundEvent::GossipMessage {
                            topic: message.topic,
                            payload: payload,
                        })
                        .await?;
                }
                _ => {}
            },
            SwarmEvent::Behaviour(Idp2pBehaviourEvent::RequestResponse(
                request_response::Event::Message { message, .. },
            )) => {
                match message {
                    request_response::Message::Request {
                        request, channel, ..
                    } => {
                        // decide
                    }
                    request_response::Message::Response {
                        request_id,
                        response,
                    } => {}
                }
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                // sleep for a second to avoid a race condition with the
                tokio::time::sleep(Duration::from_secs(1)).await;
                self.app_event_sender
                    .send(IdAppEvent::ListenOn(address.to_string()))
                    .await
                    .unwrap();
            }
            SwarmEvent::Behaviour(Idp2pBehaviourEvent::Mdns(libp2p::mdns::Event::Discovered(
                list,
            ))) => {
                for (peer_id, _multiaddr) in list {
                    //println!("mDNS discovered a new peer: {peer_id}");
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
                    //println!("mDNS discover peer has expired: {peer_id}");
                    self.swarm
                        .behaviour_mut()
                        .gossipsub
                        .remove_explicit_peer(&peer_id);
                }
            }

            _ => {}
        }
        Ok(())
    }
}
