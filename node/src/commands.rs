use crate::behaviour::IdentityGossipBehaviour;
use crate::id_message::{IdentityCommand, IdentityMessage};
use crate::wallet::Wallet;
use core::did::Identity;
use libp2p::gossipsub::IdentTopic;

pub enum Commands {
    Get {
        id: String,
    },
    Resolve {
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
    Unknown,
}

impl Commands {
    pub fn handle(&self, behaviour: &mut IdentityGossipBehaviour) {
        match self {
            Commands::Get { id } => {
                let gossipsub_topic = IdentTopic::new(id.clone());
                behaviour.gossipsub.subscribe(&gossipsub_topic).unwrap();
                behaviour.publish(id.clone(), IdentityMessage::new(IdentityCommand::Get));
            }
            Commands::Resolve { id } => {
                let identity: Identity =
                    serde_json::from_str(&behaviour.db.get(id).unwrap()).unwrap();
                println!("{:#?}", serde_json::to_string_pretty(&identity));
            }
            Commands::Create { name } => {
                let wallet = Wallet::create(name);
                let id = wallet.did.ledger.id.clone();
                let gossipsub_topic = IdentTopic::new(id.clone());
                behaviour
                    .db
                    .insert(id.clone(), serde_json::to_string(&wallet.did).unwrap());
                behaviour.gossipsub.subscribe(&gossipsub_topic).unwrap();
                println!("Created, id:{}", id);
            }
            Commands::SetProof { name, key, value } => {
                let mut wallet = Wallet::get(name);
                wallet.did.set_proof(
                    wallet.signer_secret.clone(),
                    key.as_bytes().to_vec(),
                    value.as_bytes().to_vec(),
                );
                behaviour.publish(
                    wallet.did.ledger.id.clone(),
                    IdentityMessage::new(IdentityCommand::Post(wallet.did)),
                );
            }
            Commands::Recover { name } => {
                let mut wallet = Wallet::get(name);
                let result = wallet.did.recover(wallet.recovery_secret.clone());
                wallet.recovery_secret = result.recovery_secret;
                wallet.signer_secret = result.signer_secret;
                Wallet::update(name, &wallet);
                behaviour.publish(
                    wallet.did.ledger.id.clone(),
                    IdentityMessage::new(IdentityCommand::Post(wallet.did)),
                );
            }
            Commands::ChangeDoc { name } => {
                let mut wallet = Wallet::get(name);
                let result = wallet.did.set_doc(wallet.signer_secret.clone());
                wallet.assertion_secret = result.assertion_secret;
                wallet.authentication_secret = result.authentication_secret;
                wallet.keyagreement_secret = result.keyagreement_secret;
                Wallet::update(name, &wallet);
                behaviour.publish(
                    wallet.did.ledger.id.clone(),
                    IdentityMessage::new(IdentityCommand::Post(wallet.did)),
                );
            }
            Commands::Unknown => println!("Unknown command"),
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
        "resolve" => Commands::Resolve {
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
        "change_doc" => Commands::ChangeDoc {
            name: input[1].to_string(),
        },
        _ => Commands::Unknown,
    }
}
