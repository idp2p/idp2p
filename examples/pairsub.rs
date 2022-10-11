use std::{iter, str::FromStr};

use idp2p_pairsub::{
    PairsubCodec, PairsubProtocol, PairsubRequest, PairsubRequestKind, PairsubResponse,
};
use libp2p::{
    futures::StreamExt,
    identity,
    mdns::Mdns,
    request_response::{
        ProtocolSupport, RequestResponse, RequestResponseEvent, RequestResponseMessage,
    },
    swarm::{SwarmBuilder, SwarmEvent},
    PeerId,
};
use structopt::StructOpt;
use tokio::io::AsyncBufReadExt;

use crate::pairsub_swarm::{Idp2pPairsubBehaviour, Idp2pPairsubEvent};

#[derive(Debug, StructOpt)]
#[structopt(name = "idp2pgossip", about = "Usage of idp2p gossip.")]
struct Opt {
    #[structopt(short = "p", long = "port", default_value = "43727")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let opt = Opt::from_args();
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);
    let transport = libp2p::development_transport(local_key.clone()).await?;
    let mut swarm = {
        let mdns = Mdns::new(Default::default()).await?;

        let behaviour = Idp2pPairsubBehaviour {
            ping: libp2p::ping::Ping::new(libp2p::ping::Config::new().with_keep_alive(true)),
            mdns: mdns,
            pairsub: RequestResponse::new(
                PairsubCodec(),
                iter::once((PairsubProtocol(), ProtocolSupport::Full)),
                Default::default(),
            ),
        };
        let executor = Box::new(|fut| {
            tokio::spawn(fut);
        });
        SwarmBuilder::new(transport, behaviour, local_key.public().to_peer_id())
            .executor(executor)
            .build()
    };
    swarm.listen_on(format!("/ip4/127.0.0.1/tcp/{}", opt.port).parse()?)?;

    loop {
        tokio::select! {
            line = stdin.next_line() => {
                let line = line?.expect("stdin closed");
                println!("{line}");
                swarm.behaviour_mut().pairsub.send_request(
                    &PeerId::from_str(&line)?,
                    PairsubRequest{message:PairsubRequestKind::Get, pair_id: "abc".to_owned() }
                 );
            }

            swarm_event = swarm.select_next_some() => {
                match swarm_event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                       println!("Listening on {:?}", address);
                    }
                    SwarmEvent::Behaviour(Idp2pPairsubEvent::Pairsub(event)) => {
                        if let RequestResponseEvent::Message { message, .. } = event{
                            println!("Got request: {:?}", message);
                            match message {
                                RequestResponseMessage::Request {
                                    request: _, channel, ..
                                } => {
                                    swarm
                                      .behaviour_mut()
                                      .pairsub
                                      .send_response(channel, PairsubResponse::Ok)
                                      .expect("Connection to peer sto be still open.");
                                }
                                other => { println!("Other {:?}", other); }
                            }
                        }
                    }
                    other=>{println!("Other {:?}", other);}
                }
            }
        }
    }
}

mod pairsub_swarm {
    use idp2p_pairsub::{PairsubCodec, PairsubRequest, PairsubResponse};
    use libp2p::{
        mdns::{Mdns, MdnsEvent},
        request_response::{RequestResponse, RequestResponseEvent},
        NetworkBehaviour,
    };

    #[derive(NetworkBehaviour)]
    #[behaviour(out_event = "Idp2pPairsubEvent")]
    pub struct Idp2pPairsubBehaviour {
        pub ping: libp2p::ping::Behaviour,
        pub mdns: Mdns,
        pub pairsub: RequestResponse<PairsubCodec>,
    }

    #[derive(Debug)]
    pub enum Idp2pPairsubEvent {
        Mdns(MdnsEvent),
        Ping(libp2p::ping::Event),
        Pairsub(RequestResponseEvent<PairsubRequest, PairsubResponse>),
    }

    impl From<MdnsEvent> for Idp2pPairsubEvent {
        fn from(event: MdnsEvent) -> Self {
            Idp2pPairsubEvent::Mdns(event)
        }
    }

    impl From<libp2p::ping::Event> for Idp2pPairsubEvent {
        fn from(event: libp2p::ping::Event) -> Self {
            Idp2pPairsubEvent::Ping(event)
        }
    }

    impl From<RequestResponseEvent<PairsubRequest, PairsubResponse>> for Idp2pPairsubEvent {
        fn from(event: RequestResponseEvent<PairsubRequest, PairsubResponse>) -> Self {
            Idp2pPairsubEvent::Pairsub(event)
        }
    }
}
