use crate::id_behaviour::IdentityGossipBehaviour;
use crate::id_message::{IdentityMessage, IdentityMessageType};
use idp2p_core::did::Identity;
use libp2p::gossipsub::IdentTopic;

#[derive(PartialEq, Debug, Clone)]
pub enum IdentityCommand {
    Post { did: Identity },
    Get { id: String },
}

impl IdentityCommand {
    pub fn handle(&self, behaviour: &mut IdentityGossipBehaviour) {
        match self {
            IdentityCommand::Post { did } => {
                // validate did
                // if valid and next change local store
                let id = did.id.clone();
                let gossipsub_topic = IdentTopic::new(did.id.clone());
                behaviour.gossipsub.subscribe(&gossipsub_topic).unwrap();
                behaviour.publish(
                    id.clone(),
                    IdentityMessage::new(IdentityMessageType::Post(did.clone())),
                );
            }
            IdentityCommand::Get { id } => {
                let gossipsub_topic = IdentTopic::new(id.clone());
                behaviour.gossipsub.subscribe(&gossipsub_topic).unwrap();
                behaviour.publish(id.clone(), IdentityMessage::new(IdentityMessageType::Get));
            }
        }
    }
}

pub fn handle_cmd(input: &str) -> Option<IdentityCommand> {
    let split = input.split(" ");
    let input: Vec<&str> = split.collect();
    match input[0] {
        "create-id" => {
            let name = input[1];
            let identity_result = Identity::new();
           // Wallet::create(name, identity_result.clone());
        }
        "change-doc" => {
            let name = input[1].to_string();
            
        }
        "set-proof" => {
            let name = input[1].to_string();
            let key = input[2].to_string();
            let value = input[3].to_string();
        }
        "recover" => {
            let name = input[1].to_string();
        }
        _ => {
            println!("Unknown command");
            return None;
        }
    }
    None
}

/*
use crate::id_behaviour::IdentityGossipBehaviour;
use crate::id_message::{IdentityMessageType, IdentityMessage};
use crate::wallet::Wallet;
use core::did::Identity;
use libp2p::gossipsub::IdentTopic;

impl Commands {
    pub fn handle(&self, behaviour: &mut IdentityGossipBehaviour) {
        match self {
            Commands::Get { id } => {
                let gossipsub_topic = IdentTopic::new(id.clone());
                behaviour.gossipsub.subscribe(&gossipsub_topic).unwrap();
                behaviour.publish(id.clone(), IdentityMessage::new(IdentityMessageType::Get));
            }
            Commands::Create { name } => {
                let wallet = Wallet::create(name);
                let id = wallet.did.id.clone();
                let gossipsub_topic = IdentTopic::new(id.clone());
                behaviour
                    .identities
                    .insert(id.clone(), serde_json::to_string(&wallet.did).unwrap());
                behaviour.gossipsub.subscribe(&gossipsub_topic).unwrap();
                println!("Created, {}", id);
            }
            Commands::SetProof { name, key, value } => {
                let mut wallet = Wallet::get(name);
                /*wallet.did.set_proof(
                    wallet.signer_secret.clone(),
                    key.as_bytes().to_vec(),
                    value.as_bytes().to_vec(),
                );*/
                Wallet::update(name, &wallet);
                behaviour.publish(
                    wallet.did.id.clone(),
                    IdentityMessage::new(IdentityMessageType::Post(wallet.did)),
                );
            }
            Commands::Recover { name } => {
                let mut wallet = Wallet::get(name);
               // let result = wallet.did.recover(wallet.recovery_secret.clone());
                //wallet.recovery_secret = result.recovery_secret;
                //wallet.signer_secret = result.signer_secret;
                Wallet::update(name, &wallet);
                behaviour.publish(
                    wallet.did.id.clone(),
                    IdentityMessage::new(IdentityMessageType::Post(wallet.did)),
                );
            }
            Commands::ChangeDoc { name } => {
                let mut wallet = Wallet::get(name);
                //let result = wallet.did.set_doc(wallet.signer_secret.clone());
                //wallet.assertion_secret = result.assertion_secret;
                //wallet.authentication_secret = result.authentication_secret;
                //wallet.keyagreement_secret = result.keyagreement_secret;
                Wallet::update(name, &wallet);
                behaviour.publish(
                    wallet.did.id.clone(),
                    IdentityMessage::new(IdentityMessageType::Post(wallet.did)),
                );
            }
        }
    }
}

use idp2p_core::did::CreateIdentityResult;
use std::fs::OpenOptions;
use std::fs::File;
use std::io::Read;
use idp2p_core::did::Identity;
use serde::{Deserialize, Serialize};
use idp2p_core::encode_me;

const WALLET_BASE_PATH: &str = "../target/";
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Wallet {
    pub did: Identity,
    #[serde(with = "encode_me")]
    pub signer_secret: Vec<u8>,
    #[serde(with = "encode_me")]
    pub recovery_secret: Vec<u8>,
    #[serde(with = "encode_me")]
    pub assertion_secret: Vec<u8>,
    #[serde(with = "encode_me")]
    pub authentication_secret: Vec<u8>,
    #[serde(with = "encode_me")]
    pub keyagreement_secret: Vec<u8>,
}

impl Wallet {
    pub fn create(name: &str, id_result: CreateIdentityResult) -> Wallet{
        let wallet_path = &format!("{}{}.json", WALLET_BASE_PATH, name);
        if !std::path::Path::new(wallet_path).exists() {
            std::fs::File::create(wallet_path).unwrap();
        }
        let file = OpenOptions::new().write(true).open(wallet_path).unwrap();
        let wallet = Wallet{
            did: id_result.did,
            assertion_secret: id_result.assertion_secret,
            authentication_secret: id_result.authentication_secret,
            signer_secret: id_result.signer_secret,
            recovery_secret: id_result.recovery_secret,
            keyagreement_secret: id_result.keyagreement_secret
        };
        serde_json::to_writer_pretty(&file, &wallet).unwrap();
        wallet
    }

    pub fn update(name: &str, wallet: &Wallet){
        let wallet_path = &format!("{}{}.json", WALLET_BASE_PATH, name);
        let file = OpenOptions::new().write(true).open(wallet_path).unwrap();
        serde_json::to_writer_pretty(&file, wallet).unwrap();
    }

    pub fn get(name: &str) -> Wallet{
        let wallet_path = &format!("{}{}.json", WALLET_BASE_PATH, name);
        let mut file = File::open(wallet_path).unwrap();
        let mut buff = String::new();
        file.read_to_string(&mut buff).unwrap();
        serde_json::from_str(&buff).unwrap()
    }
}


*/
