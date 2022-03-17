use idp2p_common::anyhow::Result;
use idp2p_node::node::build_gossipsub;
use idp2p_node::node::build_transport;
use libp2p::futures::StreamExt;
use libp2p::identity::Keypair;
use libp2p::swarm::SwarmEvent;
use libp2p::{
    gossipsub::{Gossipsub, GossipsubEvent, IdentTopic},
    swarm::{Swarm, SwarmBuilder},
    Multiaddr, NetworkBehaviour, PeerId,
};
use std::str::FromStr;
use structopt::StructOpt;
use tokio::io::AsyncBufReadExt;

#[derive(Debug, StructOpt)]
#[structopt(name = "idp2p", about = "Usage of idp2p.")]
struct Opt {
    #[structopt(short = "i", long = "ip", default_value = "0.0.0.0")]
    ip: String,
    #[structopt(short = "p", long = "port", default_value = "43727")]
    port: u16,
}

#[derive(Debug)]
pub enum IdentityRelayEvent {
    Gossipsub(GossipsubEvent),
}

impl From<GossipsubEvent> for IdentityRelayEvent {
    fn from(event: GossipsubEvent) -> Self {
        IdentityRelayEvent::Gossipsub(event)
    }
}

pub async fn build_swarm(local_key: Keypair) -> Swarm<IdentityRelayBehaviour> {
    let transport = build_transport(local_key.clone()).await;
    let swarm = {
        let behaviour = IdentityRelayBehaviour {
            gossipsub: build_gossipsub(),
        };
        let executor = Box::new(|fut| {
            tokio::spawn(fut);
        });
        SwarmBuilder::new(transport, behaviour, local_key.public().to_peer_id())
            .executor(executor)
            .build()
    };
    return swarm;
}

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "IdentityRelayEvent")]
pub struct IdentityRelayBehaviour {
    pub gossipsub: Gossipsub,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();
    let local_key = Keypair::generate_ed25519();
    let mut swarm = build_swarm(local_key.clone()).await;
    swarm.listen_on(format!("/ip4/{}/tcp/{}", opt.ip, opt.port).parse()?)?;

    loop {
        tokio::select! {
            line = stdin.next_line() => {
                let line = line?.expect("stdin closed");
                run_command(&line, &mut swarm)?;
            }
            swarm_event = swarm.select_next_some() => {
                match swarm_event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening on {:?}", address);
                    }
                    SwarmEvent::Behaviour(IdentityRelayEvent::Gossipsub(event)) =>{
                        println!("Got message: {:?}", event);
                        //swarm.behaviour_mut().handle_gossip_event(&owner, event).await;
                    }
                    other => { println!("{:?}", other); }
                }
            }
        }
    }
}

fn run_command(input: &str, swarm: &mut Swarm<IdentityRelayBehaviour>) -> Result<()> {
    let split = input.split(" ");
    let input: Vec<&str> = split.collect();
    match input[0] {
        "connect" => {
            let to_dial = format!("/ip4/{}/tcp/{}", input[1], input[2]);
            let addr: Multiaddr = to_dial.parse().unwrap();
            let peer_id = PeerId::from_str(input[3])?;
            swarm.dial(addr)?;
            swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
            swarm
                .behaviour_mut()
                .gossipsub
                .subscribe(&IdentTopic::new(input[3]))?;
        }
        "subscribe" => {
            swarm
                .behaviour_mut()
                .gossipsub
                .subscribe(&IdentTopic::new(input[2]))?;
            swarm
                .behaviour_mut()
                .gossipsub
                .publish(IdentTopic::new(input[1]), input[2].as_bytes())
                .unwrap();
        }
        "publish" => {
            let topic = IdentTopic::new(input[1]);
            swarm
                .behaviour_mut()
                .gossipsub
                .publish(topic, input[2].as_bytes())
                .unwrap();
        }
        _ => {}
    }
    Ok(())
}
