use std::{iter, str::FromStr, sync::Arc};

use idp2p_common::{anyhow::Result, secret::EdSecret, log};
use idp2p_core::protocol::{
    gossip::{build_gossipsub, IdGossip, IdGossipMessage, IdGossipMessagePayload},
    req_res::{
        IdCodec, IdProtocol, IdRequest, IdRequestNodeMessage, IdRequestPayload, IdResponse,
        IdResponsePayload, IdResponsePayloadOk,
    },
};
use libp2p::{
    core::{self, muxing::StreamMuxerBox, transport::Boxed},
    dns,
    gossipsub::{Gossipsub, GossipsubEvent, IdentTopic},
    identity::{ed25519::SecretKey, Keypair},
    mdns::{Mdns, MdnsEvent},
    mplex, noise,
    request_response::{
        ProtocolSupport, RequestResponse, RequestResponseEvent, RequestResponseMessage,
    },
    swarm::SwarmBuilder,
    tcp, websocket, yamux, Multiaddr, NetworkBehaviour, PeerId, Swarm, Transport,
};

use crate::store::{HandleGetResult, NodeStore};

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "IdentityNodeEvent")]
pub struct IdentityNodeBehaviour {
    mdns: Mdns,
    pub req_res: RequestResponse<IdCodec>,
    pub gossipsub: Gossipsub,
    #[behaviour(ignore)]
    pub store: Arc<NodeStore>,
}

#[derive(Debug)]
pub enum IdentityNodeEvent {
    Gossipsub(GossipsubEvent),
    RequestResponse(RequestResponseEvent<IdRequest, IdResponse>),
    Mdns(MdnsEvent),
}

impl From<GossipsubEvent> for IdentityNodeEvent {
    fn from(event: GossipsubEvent) -> Self {
        IdentityNodeEvent::Gossipsub(event)
    }
}

impl From<RequestResponseEvent<IdRequest, IdResponse>> for IdentityNodeEvent {
    fn from(event: RequestResponseEvent<IdRequest, IdResponse>) -> Self {
        IdentityNodeEvent::RequestResponse(event)
    }
}

impl From<MdnsEvent> for IdentityNodeEvent {
    fn from(event: MdnsEvent) -> Self {
        IdentityNodeEvent::Mdns(event)
    }
}

type ReqResEvent = RequestResponseEvent<IdRequest, IdResponse>;
impl IdentityNodeBehaviour {
    pub fn handle_mdns_event(&mut self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(list) => {
                for (peer, _) in list {
                    self.gossipsub.add_explicit_peer(&peer);
                }
            }
            MdnsEvent::Expired(list) => {
                for (peer, _) in list {
                    if !self.mdns.has_node(&peer) {
                        self.gossipsub.remove_explicit_peer(&peer);
                    }
                }
            }
        }
    }

    pub async fn handle_gossip_event(&mut self, event: GossipsubEvent) -> Result<()> {
        if let GossipsubEvent::Message {
            propagation_source: _,
            message_id: _,
            message,
        } = event
        {
            let topic = message.topic.to_string();
            if idp2p_common::is_idp2p(&topic) {
                let message = IdGossipMessage::from_bytes(&message.data)?;
                idp2p_common::log::info!(
                    "Message received topic: {} message: {:?}",
                    topic,
                    message
                );
                match &message.payload {
                    IdGossipMessagePayload::Get => {
                        let get_result = self.store.handle_get(&topic).await?;
                        match get_result {
                            HandleGetResult::Publish(id) => {
                                if let Some(did) = self.store.get_did_for_post(&id) {
                                    self.gossipsub.publish_post(did)?;
                                }
                            }
                            HandleGetResult::WaitAndPublish(id) => {
                                let (tx, rx) = tokio::sync::oneshot::channel::<String>();
                                tokio::spawn(async move {
                                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                                    tx.send(id).unwrap();
                                });
                                if let Some(did) = self.store.get_did_for_post(&rx.await?) {
                                    self.gossipsub.publish_post(did)?;
                                }
                            }
                        }
                    }
                    IdGossipMessagePayload::Post { digest, identity } => {
                        self.store.handle_post(digest, identity).await?;
                    }
                    // Pass message to all client peers
                    IdGossipMessagePayload::Jwm { jwm } => {
                        if let Some(client) = self.store.get_client(&topic) {
                            for peer_id in client.peers {
                                let peer = PeerId::from_str(&peer_id)?;
                                let req =
                                    IdRequest(IdRequestPayload::WalletMessage(jwm.to_owned()));
                                self.req_res.send_request(&peer, req);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn handle_client_request(&mut self, event: ReqResEvent) -> Result<()> {
        if let RequestResponseEvent::Message { message, peer } = event {
            match message {
                RequestResponseMessage::Request {
                    request, channel, ..
                } => {
                    let mut response = IdResponse(IdResponsePayload::Ok(IdResponsePayloadOk::None));
                    match request.0 {
                        IdRequestPayload::Register {
                            identity,
                            subscriptions,
                            proof,
                        } => {
                            log::info!("{proof}"); // check proof
                            if self.store.get_client(&identity.id).is_none() {
                                self.store.create(identity.clone()).await;
                            }
                            let mut client = self.store.get_client(&identity.id).unwrap();
                            client.peers.push(peer.to_base58());
                            for subsciption in subscriptions {
                                if !client.subscriptions.contains(&subsciption) {
                                    self.gossipsub.subscribe_to(&subsciption)?;
                                    client.subscriptions.insert(subsciption);
                                }
                            }
                        }
                        IdRequestPayload::NodeMessage { id, message } => {
                            if let Some(mut client) = self.store.get_client(&id) {
                                match message {
                                    IdRequestNodeMessage::Get(id) => {
                                        if let Some(did) = self.store.get_did(&id){
                                            response = IdResponse(IdResponsePayload::Ok(IdResponsePayloadOk::GetResult(did)));
                                        }else{
                                            response = IdResponse(IdResponsePayload::Error(
                                                "IdentityNotfound".to_owned(),
                                            ));
                                        }
                                    }
                                    IdRequestNodeMessage::Publish { id, jwm } => {
                                        if client.subscriptions.contains(&id) {
                                            let msg = IdGossipMessage::new_jwm(&jwm);
                                            let topic = IdentTopic::new(&id);
                                            self.gossipsub.publish(topic, msg.to_bytes()?)?;
                                        } else {
                                            response = IdResponse(IdResponsePayload::Error(
                                                "InvalidClient".to_owned(),
                                            ));
                                        }
                                    }
                                    IdRequestNodeMessage::Subscribe(id) => {
                                        if !client.subscriptions.contains(&id) {
                                            self.gossipsub.subscribe_to(&id)?;
                                            client.subscriptions.insert(id);
                                        }
                                    }
                                }
                            } else {
                                response = IdResponse(IdResponsePayload::Error(
                                    "InvalidClient".to_owned(),
                                ));
                                log::error!("No client found");
                            }
                        }
                        IdRequestPayload::WalletMessage(_) => {
                            response = IdResponse(IdResponsePayload::Error(
                                "InvalidMessageType".to_owned(),
                            ));
                            log::error!("Wallet message is not supported on node");
                        }
                    }
                    self.req_res
                        .send_response(channel, response)
                        .expect("No connection");
                }
                _ => {}
            }
        }
        Ok(())
    }
}

pub struct NodeOptions {
    listen: String,
    to_dial: Option<String>,
}

impl NodeOptions {
    pub fn new() -> Self {
        Self {
            to_dial: None,
            listen: "/ip4/0.0.0.0/tcp/43727".to_owned(),
        }
    }

    pub fn new_with_listen(listen: &str) -> Self {
        Self {
            to_dial: None,
            listen: listen.to_owned(),
        }
    }
}

async fn build_transport(local_key: Keypair) -> Boxed<(PeerId, StreamMuxerBox)> {
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);
    let transport = {
        let tcp = tcp::TcpConfig::new().nodelay(true);
        let dns_tcp = dns::DnsConfig::system(tcp).await.unwrap();
        let ws_dns_tcp = websocket::WsConfig::new(dns_tcp.clone());
        dns_tcp.or_transport(ws_dns_tcp)
    };
    let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
        .into_authentic(&local_key)
        .expect("Signing libp2p-noise static DH keypair failed.");
    let boxed = transport
        .upgrade(core::upgrade::Version::V1)
        .authenticate(noise::NoiseConfig::xx(noise_keys).into_authenticated())
        .multiplex(core::upgrade::SelectUpgrade::new(
            yamux::YamuxConfig::default(),
            mplex::MplexConfig::default(),
        ))
        .timeout(std::time::Duration::from_secs(20))
        .boxed();
    boxed
}

pub async fn build_swarm(options: NodeOptions) -> Result<Swarm<IdentityNodeBehaviour>> {
    let secret_key = SecretKey::from_bytes(EdSecret::new().to_bytes())?;
    let local_key = Keypair::Ed25519(secret_key.into());
    let transport = build_transport(local_key.clone()).await;
    let id_store = Arc::new(NodeStore::new());
    let mut swarm = {
        let req_res = RequestResponse::new(
            IdCodec(),
            iter::once((IdProtocol(), ProtocolSupport::Full)),
            Default::default(),
        );
        let mdns = Mdns::new(Default::default()).await?;

        let behaviour = IdentityNodeBehaviour {
            gossipsub: build_gossipsub(),
            req_res: req_res,
            mdns: mdns,
            store: id_store,
        };
        let executor = Box::new(|fut| {
            tokio::spawn(fut);
        });
        SwarmBuilder::new(transport, behaviour, local_key.public().to_peer_id())
            .executor(executor)
            .build()
    };
    swarm.listen_on(options.listen.parse()?)?;
    if let Some(connect) = options.to_dial {
        let split: Vec<&str> = connect.split("/").collect();
        let to_dial = format!("/ip4/{}/tcp/{}", split[2], split[4]);
        let addr: Multiaddr = to_dial.parse().unwrap();
        let peer_id = PeerId::from_str(split[6])?;
        swarm.dial(addr)?;
        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
    }
    Ok(swarm)
}
