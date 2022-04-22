use std::{iter, str::FromStr, sync::Arc};

use idp2p_common::{anyhow::Result, ed_secret::EdSecret, log};
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
    identity::{ed25519::SecretKey, Keypair},
    mplex, noise,
    request_response::{
        ProtocolSupport, RequestResponse, RequestResponseEvent, RequestResponseMessage,
    },
    swarm::SwarmBuilder,
    tcp, websocket, yamux, Multiaddr, NetworkBehaviour, PeerId, Swarm, Transport,
};

use crate::store::{HandleGetResult, NodeStore};

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "IdentityClientEvent")]
pub struct IdentityClientBehaviour {
    pub req_res: RequestResponse<IdCodec>,
    #[behaviour(ignore)]
    pub store: Arc<WalletStore>,
}

#[derive(Debug)]
pub enum IdentityClientEvent {
    RequestResponse(RequestResponseEvent<IdRequest, IdResponse>),
}

impl From<RequestResponseEvent<IdRequest, IdResponse>> for IdentityClientEvent {
    fn from(event: RequestResponseEvent<IdRequest, IdResponse>) -> Self {
        IdentityClientEvent::RequestResponse(event)
    }
}

type ReqResEvent = RequestResponseEvent<IdRequest, IdResponse>;
impl IdentityNodeBehaviour {
    pub async fn handle_client_request(&mut self, event: ReqResEvent) -> Result<()> {
        if let RequestResponseEvent::Message { message, peer } = event {
            match message {
                RequestResponseMessage::Request {
                    request, channel, ..
                } => {
                    let mut response = IdResponse(IdResponsePayload::Ok(IdResponsePayloadOk::None));
                    match request.0 {
                        IdRequestPayload::WalletMessage(_) => {
                            response = IdResponse(IdResponsePayload::Error(
                                "InvalidMessageType".to_owned(),
                            ));
                            log::error!("Wallet message is not supported on node");
                        }
                        _ => {
                            response = IdResponse(IdResponsePayload::Error(
                                "InvalidMessageType".to_owned(),
                            ));
                            log::error!("Node message is not supported on client");
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

        let behaviour = IdentityNodeBehaviour {
            req_res: req_res,
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
