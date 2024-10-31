use axum::{extract::State, routing::get, Router};
use dotenv::dotenv;
use idp2p_common::store::KvStore;
use libp2p::{
    noise, ping,
    request_response::{self, ProtocolSupport},
    swarm::NetworkBehaviour,
    tcp, yamux, Multiaddr, StreamProtocol,
};
use serde::{Deserialize, Serialize};
use std::{error::Error, sync::Arc, time::Duration};
use tokio::{io::AsyncBufReadExt, select, task};

async fn root_handler(State(state): State<Arc<AppState>>) -> String {
    std::str::from_utf8(&state.kv.get("key").unwrap().unwrap()).unwrap().to_owned()
}

fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(root_handler))
        .with_state(app_state)
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct FileRequest(String);
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FileResponse(Vec<u8>);
struct AppState {
    kv: Arc<dyn KvStore>,
}
#[derive(NetworkBehaviour)]
struct Behaviour {
    request_response: request_response::cbor::Behaviour<FileRequest, FileResponse>,
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    env_logger::init();
    let kv = Arc::new(idp2p_common::store::InMemoryKvStore::new());
    kv.put("key", b"abc").unwrap();
    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| Behaviour {
            request_response: request_response::cbor::Behaviour::new(
                [(
                    StreamProtocol::new("/file-exchange/1"),
                    ProtocolSupport::Full,
                )],
                request_response::Config::default(),
            ),
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    swarm.listen_on("/ip4/0.0.0.0/tcp/43727".parse().unwrap())?;
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();
    let akv = kv.clone();
    let axum_server = task::spawn(async move {
        let app = create_router(Arc::new(AppState { kv: akv }));
        let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();

        axum::serve(listener, app).await.unwrap();
    });

    loop {
        select! {
            Ok(Some(line)) = stdin.next_line() => {
                kv.clone().put("key", line.as_bytes()).unwrap();
                println!("Publish error: {line:?}");
            }
        }
    }
}
/*use libp2p::{
    swarm::NetworkBehaviour,
    gossipsub::{Behaviour as GossipsubBehaviour, Event as GossipsubEvent},
    mdns::{Event as MdnsEvent, tokio::Behaviour as MdnsBehaviour},
    request_response::{cbor::Behaviour as CborBehaviour, Event as RequestResponseEvent},
};
use serde::{Deserialize, Serialize};

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "Idp2pNodeEvent")]
pub struct IdGossipBehaviour {
    pub id_mdns: MdnsBehaviour,
    pub id_gossipsub: GossipsubBehaviour,
    pub id_resolve: CborBehaviour<IdDocument, ()>,
    pub id_message: CborBehaviour<IdDirectMessage, ()>,
    pub id_request: CborBehaviour<IdRequest, ()>,
}

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "Idp2pNodeEvent")]
pub struct Idp2pNodeBehaviour {
    pub id_mdns: MdnsBehaviour,
    pub id_gossipsub: GossipsubBehaviour<m>,
    pub id_reqres: CborBehaviour<IdRequest, IdResponse>,
}

#[derive(Debug)]
pub enum Idp2pNodeEvent {
    Mdns(MdnsEvent),
    Gossipsub(GossipsubEvent),
    IdRequestResponse(RequestResponseEvent<IdRequest, IdResponse>),
}

impl From<MdnsEvent> for Idp2pNodeEvent {
    fn from(event: MdnsEvent) -> Self {
        Idp2pNodeEvent::Mdns(event)
    }
}

impl From<GossipsubEvent> for Idp2pNodeEvent {
    fn from(event: GossipsubEvent) -> Self {
        Idp2pNodeEvent::Gossipsub(event)
    }
}

impl From<RequestResponseEvent<IdRequest, IdResponse>> for Idp2pNodeEvent {
    fn from(event: RequestResponseEvent<NodeRequest, NodeResponse>) -> Self {
        Idp2pNodeEvent::NodeRequestResponse(event)
    }
}

impl From<RequestResponseEvent<VerifierRequest, VerifierResponse>> for Idp2pNodeEvent {
    fn from(event: RequestResponseEvent<VerifierRequest, VerifierResponse>) -> Self {
        Idp2pNodeEvent::VerifierRequestResponse(event)
    }
}

impl Idp2pNodeBehaviour {
    pub fn handle_mdns_event(&mut self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(list) => {
                for (peer, _) in list {
                    self.gossipsub.add_explicit_peer(&peer);
                }
            }
            MdnsEvent::Expired(list) => {
                for (peer, _) in list {
                    if !self.mdns.discovered_nodes().any(|p| p == &peer) {
                        self.gossipsub.remove_explicit_peer(&peer);
                    }
                }
            }
        }
    }

    pub fn handle_gossip_event(&mut self, event: GossipsubEvent) {
        match event {
            GossipsubEvent::Message {
                propagation_source,
                message_id,
                message,
            } => if message.topic.as_str().len() == 0 {
                    match message.payload {
                        IdGossipMessageKind::Resolve => {
                            // if the node is provider for the identity
                            // publish the identity document
                        },
                        IdGossipMessageKind::Provide { provider } => {
                            // if the identity doesn't exist
                            // save the identity
                            // add it to the list of providers
                        },
                        IdGossipMessageKind::NotifyEvent { event } => {
                            // verify event and update state
                        },
                        IdGossipMessageKind::NotifyMessage { message_id } => {
                            // store message
                        },
                    }
                 },
            _ => {}
        }
    }
}*/
