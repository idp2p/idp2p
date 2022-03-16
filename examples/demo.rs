use colored::Colorize;
use idp2p_common::ed_secret::EdSecret;
use idp2p_core::did::Identity;
use idp2p_didcomm::jpm::Jpm;
use idp2p_didcomm::jwe::Jwe;
use idp2p_didcomm::jwm::Jwm;
use idp2p_didcomm::jws::Jws;
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

pub fn handle_jwm(jwm: &str, behaviour: &mut IdentityNodeBehaviour) {
    let secret_str = std::env::var("secret").unwrap();
    let dec_secret = EdSecret::from_str(&secret_str).unwrap();
    let jwe: Jwe = idp2p_common::serde_json::from_str(jwm).unwrap();
    let json = jwe.decrypt(dec_secret).unwrap();
    let jws: Jws = idp2p_common::serde_json::from_str(&json).unwrap();
    let jpm: Jpm = idp2p_common::base64url::decode(&jws.payload).unwrap();
    let from = behaviour.id_store.get_did(&jpm.from);
    jws.verify(from).unwrap();
    let rec_mes = format!("Received message {}", jpm.body.to_string().green());
    println!("{rec_mes}");
}

fn run_command(input: &str, behaviour: &mut IdentityNodeBehaviour) {
    let split = input.split(" ");
    let input: Vec<&str> = split.collect();
    match input[0] {
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
        "send-message" => {
            let secret_str = std::env::var("secret").unwrap();
            let secret = EdSecret::from_str(&secret_str).unwrap();
            let from_did = behaviour.id_store.get_owner();
            let to_did = behaviour.id_store.get_did(input[1]);
            let jwm = Jwm::new(from_did.clone(), to_did, input[2]);
            let jwe = jwm.seal(secret).unwrap();
            let json = idp2p_common::serde_json::to_string(&jwe).unwrap();
            let mes_payload = IdentityMessagePayload::Jwm { message: json };
            let mes = IdentityMessage::new(mes_payload);
            let topic = IdentTopic::new(input[1]);
            let data = idp2p_common::serde_json::to_vec(&mes).unwrap();
            behaviour.gossipsub.publish(topic, data).unwrap();
        }
        _ => {}
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();
    let (tx, mut rx) = channel::<IdentityEvent>(100);
    let secret = EdSecret::new();
    let did = Identity::from_secret(secret.clone());
    std::env::set_var("secret", idp2p_common::encode(&secret.to_bytes()));
    println!("Created: {}", did.id);
    let options = SwarmOptions {
        port: opt.port,
        owner: did.clone(),
        event_sender: tx.clone(),
    };
    let mut swarm = build_swarm(options).await?;
    let topic = IdentTopic::new(&did.id);
    swarm.behaviour_mut().gossipsub.subscribe(&topic).unwrap();
    swarm.behaviour_mut().id_store.push(did);
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
                        IdentityEvent::Published { id } => { swarm.behaviour_mut().post(&id); }
                        IdentityEvent::ReceivedJwm { id, jwm  } => { handle_jwm(&jwm, swarm.behaviour_mut()) }
                        _ => {  }
                    }
                }
            }
        }
    }
}
