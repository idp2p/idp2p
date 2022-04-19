use std::{iter, str::FromStr};

use idp2p_common::anyhow::Result;
use idp2p_core::protocol::{IdNodeRequestPayload, IdResponsePayload};
use idp2p_node::req_res::{IdNodeCodec, IdNodeProtocol, IdNodeRequest, IdNodeResponse};
use libp2p::{
    futures::StreamExt,
    mdns::{Mdns, MdnsEvent},
    request_response::{
        ProtocolSupport, RequestResponse, RequestResponseEvent, RequestResponseMessage,
    },
    swarm::SwarmEvent,
    NetworkBehaviour, PeerId,
};
use tokio::io::AsyncBufReadExt;

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "IdentityNodeEvent")]
pub struct IdentityNodeBehaviour {
    mdns: Mdns,
    req_res: RequestResponse<IdNodeCodec>,
}

#[derive(Debug)]
pub enum IdentityNodeEvent {
    Mdns(MdnsEvent),
    RequestResponse(RequestResponseEvent<IdNodeRequest, IdNodeResponse>),
}

impl From<MdnsEvent> for IdentityNodeEvent {
    fn from(event: MdnsEvent) -> Self {
        IdentityNodeEvent::Mdns(event)
    }
}

impl From<RequestResponseEvent<IdNodeRequest, IdNodeResponse>> for IdentityNodeEvent {
    fn from(event: RequestResponseEvent<IdNodeRequest, IdNodeResponse>) -> Self {
        IdentityNodeEvent::RequestResponse(event)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let local_key = libp2p::identity::Keypair::generate_ed25519();
    let local_peer_id = libp2p::PeerId::from(local_key.public());
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();
    println!("Local peer id: {:?}", local_peer_id);
    let transport = libp2p::development_transport(local_key).await?;
    let behaviour = IdentityNodeBehaviour {
        mdns: Mdns::new(Default::default()).await?,
        req_res: RequestResponse::new(
            IdNodeCodec(),
            iter::once((IdNodeProtocol(), ProtocolSupport::Full)),
            Default::default(),
        ),
    };
    let mut swarm = libp2p::Swarm::new(transport, behaviour, local_peer_id);
    swarm.listen_on("/ip4/127.0.0.1/tcp/0".parse()?)?;
    loop {
        tokio::select! {
            line = stdin.next_line() => {
                let line = line?.expect("stdin closed");
                swarm.behaviour_mut().req_res.send_request(&PeerId::from_str(&line)?, IdNodeRequest(IdNodeRequestPayload::Register));
            }
            swarm_event = swarm.select_next_some() => {
                match swarm_event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                       println!("Listening on {:?}", address);
                    }
                    SwarmEvent::Behaviour(IdentityNodeEvent::RequestResponse(event)) => {
                        if let RequestResponseEvent::Message { message, .. } = event{
                            println!("Got request: {:?}", message);
                            match message {
                                RequestResponseMessage::Request {
                                    request: _, channel, ..
                                } => {
                                    swarm
                                      .behaviour_mut()
                                      .req_res
                                      .send_response(channel, IdNodeResponse(IdResponsePayload::Ok))
                                      .expect("Connection to peer sto be still open.");
                                }
                                _=>{}
                            }
                        }
                    }
                    _ => {  }
                }
            }
        }
    }
}
