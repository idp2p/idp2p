use std::fs::OpenOptions;
use std::fs::File;
use std::io::Read;
use core::did::Identity;
use serde::{Deserialize, Serialize};
use core::encode_me;

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
    pub fn create(name: &str) -> Wallet{
        let wallet_path = &format!("{}{}.json", WALLET_BASE_PATH, name);
        if !std::path::Path::new(wallet_path).exists() {
            std::fs::File::create(wallet_path).unwrap();
        }
        let file = OpenOptions::new().write(true).open(wallet_path).unwrap();
        let r = Identity::new();
        let wallet = Wallet{
            did: r.did,
            assertion_secret: r.assertion_secret,
            authentication_secret: r.authentication_secret,
            signer_secret: r.signer_secret,
            recovery_secret: r.recovery_secret,
            keyagreement_secret: r.keyagreement_secret
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
