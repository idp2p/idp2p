use crate::behaviour::IdentityGossipBehaviour;
use crate::wallet::Wallet;
use libp2p::gossipsub::IdentTopic;
use rand::prelude::*;

pub enum Commands {
    Get {
        id: String,
    },
    Create {
        name: String,
    },
    SetProof {
        name: String,
        key: String,
        value: String,
    },
    ChangeDoc {
        name: String,
    },
    Recover {
        name: String,
    },
}

fn post(behaviour: &mut IdentityGossipBehaviour, id: String, data: String) {
    let gossipsub_topic = IdentTopic::new(id.clone());
    behaviour
        .gossipsub
        .publish(gossipsub_topic.clone(), data.as_bytes())
        .unwrap();
}

impl Commands {
    pub fn handle(&self, behaviour: &mut IdentityGossipBehaviour) {
        match self {
            Commands::Get { id } => {
                let mut key_data = [0u8; 32];
                let mut key_rng = thread_rng();
                key_rng.fill_bytes(&mut key_data);
                post(
                    behaviour,
                    id.clone(),
                    format!("get {}", String::from_utf8_lossy(&key_data)),
                );
            }
            Commands::Create { name } => {
                let wallet = Wallet::create(name);
                let id = wallet.did.ledger.id.clone();
                let gossipsub_topic = IdentTopic::new(id.clone());
                behaviour
                    .db
                    .insert(id.clone(), serde_json::to_string(&wallet.did).unwrap());
                behaviour.gossipsub.subscribe(&gossipsub_topic).unwrap();
                let gossipsub_topic = IdentTopic::new(id.clone());
                let _ = behaviour.gossipsub.publish(
                    gossipsub_topic.clone(),
                    serde_json::to_string(&wallet.did).unwrap().as_bytes(),
                );
                println!("Created, id:{}", id);
            }
            Commands::SetProof { name, key, value } => {
                let mut wallet = Wallet::get(name);
                wallet.did.set_proof(
                    wallet.signer_secret.clone(),
                    key.as_bytes().to_vec(),
                    value.as_bytes().to_vec(),
                );
                post(
                    behaviour,
                    wallet.did.ledger.id.clone(),
                    serde_json::to_string(&wallet.did).unwrap(),
                );
            }
            Commands::Recover { name } => {
                post(behaviour, name.clone(), "post".to_string());
            }
            Commands::ChangeDoc { name } => {
                post(behaviour, name.clone(), "post".to_string());
            }
        }
    }
}

pub fn get_command(input: &str) -> Commands {
    let split = input.split(" ");
    let input: Vec<&str> = split.collect();
    match input[0] {
        "get" => Commands::Get {
            id: input[1].to_string(),
        },
        "create" => Commands::Create {
            name: input[1].to_string(),
        },
        "set_proof" => Commands::SetProof {
            name: input[1].to_string(),
            key: input[2].to_string(),
            value: input[3].to_string(),
        },
        "recover" => Commands::Recover {
            name: input[1].to_string(),
        },
        "change_doc" => Commands::Recover {
            name: input[1].to_string(),
        },
        _ => panic!(""),
    }
}
