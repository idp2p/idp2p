use crate::behaviour::IdentityGossipBehaviour;
use crate::wallet::Wallet;
use libp2p::gossipsub::IdentTopic;
use serde_json::json;

pub enum Commands {
    Get {
        id: String,
    },
    Create {
        name: String,
    },
    AddProof {
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
    if behaviour.db.contains_key(&id) {
        println!("{:#?}", behaviour.db.get(&id).unwrap())
    } else {
        behaviour.gossipsub.subscribe(&gossipsub_topic).unwrap();
        behaviour
            .gossipsub
            .publish(gossipsub_topic.clone(), data.as_bytes())
            .unwrap();
    }
}

impl Commands {
    pub fn handle(&self, behaviour: &mut IdentityGossipBehaviour) {
        match self {
            Commands::Get { id } => {
                post(behaviour, id.clone(), "get".to_string());
            }
            Commands::Create { name } => {
                let wallet = Wallet::create(name.to_string());
                let id = wallet.did.ledger.id;
                let gossipsub_topic = IdentTopic::new(id.clone());
                behaviour.db.insert(id.clone(), name.to_string());
                behaviour.gossipsub.subscribe(&gossipsub_topic).unwrap();
                println!("Created, id:{}", id);
            }
            Commands::AddProof { name, key, value } => {
                // let did = get_did(name)
                // update did
                // behaviour.db.get(k: &Q)
                let cmd = json!({"type": "add_proof", "key": key, "value": value});
                let data = serde_json::to_string(&cmd).unwrap();
                post(behaviour, name.clone(), data);
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
        "add_proof" => Commands::AddProof {
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
