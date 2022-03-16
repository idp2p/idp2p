use idp2p_common::ed_secret::EdSecret;
use idp2p_core::did::Identity;
use idp2p_node::message::{IdentityMessage, IdentityMessagePayload};
use idp2p_node::node::build_swarm;
use idp2p_node::node::IdentityNodeBehaviour;
use idp2p_node::node::IdentityNodeEvent;
use idp2p_node::node::SwarmOptions;
use idp2p_node::IdentityEvent;
use libp2p::futures::StreamExt;
use libp2p::gossipsub::IdentTopic;
use libp2p::swarm::SwarmEvent;
use structopt::StructOpt;
use tokio::io::AsyncBufReadExt;
use tokio::sync::mpsc::channel;

#[derive(Debug, StructOpt)]
#[structopt(name = "idp2p", about = "Usage of idp2p.")]
struct Opt {
    #[structopt(short = "p", long = "port", default_value = "43727")]
    port: u16,
    #[structopt(short = "a", long = "address")]
    address: Option<String>,
}

fn run_command(input: &str, behaviour: &mut IdentityNodeBehaviour) {
    let split = input.split(" ");
    let input: Vec<&str> = split.collect();
    match input[0] {
        "create" => {
            let secret = EdSecret::new();
            std::env::set_var(
                format!("{}_secret", input[1]),
                idp2p_common::encode(&secret.to_bytes()),
            );
            let did = Identity::from_secret(secret);
            let topic = IdentTopic::new(&did.id);
            behaviour.gossipsub.subscribe(&topic).unwrap();
            println!("Created: {}", did.id);
            behaviour.id_store.push(did);
        }
        "get" => {
            let topic = IdentTopic::new(input[1]);
            behaviour.gossipsub.subscribe(&topic).unwrap();
            let message = IdentityMessage::new(IdentityMessagePayload::Get);
            let data = idp2p_common::serde_json::to_vec(&message).unwrap();
            behaviour.gossipsub.publish(topic, data).unwrap();
        }
        "resolve" => {
            let did = behaviour.id_store.get_did(input[1]);
            println!("Resolved: {:?}", did);
        }
        "send" => {}
        _ => {}
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();
    let (tx, mut rx) = channel::<IdentityEvent>(100);
    let options = SwarmOptions {
        port: opt.port,
        owner: Identity::new(&vec![], &vec![]),
        event_sender: tx.clone(),
    };
    let mut swarm = build_swarm(options).await?;
    if opt.address.is_some() {
        println!("{}", opt.address.unwrap());
    }
    loop {
        tokio::select! {
            line = stdin.next_line() => {
                let line = line?.expect("stdin closed");
                run_command(&line, swarm.behaviour_mut());
            }
            swarm_event = swarm.select_next_some() => {
                match swarm_event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening on {:?}", address);
                    }
                    SwarmEvent::Behaviour(IdentityNodeEvent::Gossipsub(event)) =>{
                       swarm.behaviour_mut().handle_gossipevent(event).await;
                    }
                    SwarmEvent::Behaviour(IdentityNodeEvent::Mdns(event)) =>{
                        swarm.behaviour_mut().handle_mdnsevent(event);
                    }
                    _ => {}
                }
            }
            event = rx.recv() => {
                if let Some(event) = event{
                    match event{
                        IdentityEvent::Published{id} => {swarm.behaviour_mut().post(&id);}
                        _ => { println!("Event: {:?}", event); }
                    }
                }
            }
        }
    }
}
