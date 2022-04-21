use idp2p_common::anyhow::Result;
use idp2p_common::ed_secret::EdSecret;
use idp2p_core::did::identity::Identity;
use idp2p_core::protocol::gossip::IdGossip;
use idp2p_node::swarm::{build_swarm, IdentityNodeEvent, NodeOptions};
use libp2p::futures::StreamExt;
use libp2p::swarm::SwarmEvent;
use structopt::StructOpt;
use tokio::io::AsyncBufReadExt;

#[derive(Debug, StructOpt)]
#[structopt(name = "idp2p", about = "Usage of idp2p.")]
struct Opt {
    #[structopt(short = "p", long = "port", default_value = "43727")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    env_logger::init();
    let opt = Opt::from_args();
    let node_options = NodeOptions::new_with_listen(&format!("/ip4/127.0.0.1/tcp/{}", opt.port));
    let mut swarm = build_swarm(node_options).await?;
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();
    loop {
        tokio::select! {
            line = stdin.next_line() => {
                let input = line?.unwrap();
                let input: Vec<&str> = input.split(" ").collect();
                match input[0]{
                    "create" => {
                        let did = Identity::from_secret(EdSecret::new());
                        let id = did.id.clone();
                        swarm.behaviour_mut().store.create(did).await;
                        swarm.behaviour_mut().gossipsub.subscribe_to(&id)?;
                    }
                    "get" =>{
                        let id = input[1];
                        swarm.behaviour_mut().gossipsub.subscribe_to(&id)?;
                        swarm.behaviour_mut().gossipsub.publish_get(&id)?;
                    }
                    "resolve" =>{
                        let did = swarm.behaviour_mut().store.get_did(input[1]);
                        idp2p_common::log::info!("{}", idp2p_common::serde_json::to_string_pretty(&did)?);
                    }
                    _ => println!("Unknown command")
                }
            }
            swarm_event = swarm.select_next_some() => {
                match swarm_event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening on {:?}", address);
                    }
                    SwarmEvent::Behaviour(IdentityNodeEvent::Mdns(event)) =>{
                        swarm.behaviour_mut().handle_mdns_event(event);
                    }
                    SwarmEvent::Behaviour(IdentityNodeEvent::Gossipsub(event)) =>{
                        swarm.behaviour_mut().handle_gossip_event(event).await?;
                        // if jwm send it to client
                    },
                    SwarmEvent::Behaviour(IdentityNodeEvent::RequestResponse(event)) => {
                        swarm.behaviour_mut().handle_client_request(event).await?;
                    }
                    _ => {  }
                }
            }
        }
    }
}
