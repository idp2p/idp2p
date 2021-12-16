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
