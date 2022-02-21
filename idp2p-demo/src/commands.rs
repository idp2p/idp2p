use crate::FileStore;
use idp2p_common::anyhow::*;
use idp2p_common::base64url;
use idp2p_common::ed_secret::EdSecret;
use idp2p_common::serde_json;
use idp2p_didcomm::jpm::Jpm;
use idp2p_didcomm::jwe::Jwe;
use idp2p_didcomm::jwm::Jwm;
use idp2p_didcomm::jws::Jws;
use idp2p_node::behaviour::IdentityGossipBehaviour;
use idp2p_node::message::{IdentityMessage, IdentityMessagePayload};
use idp2p_node::store::IdStore;
use idp2p_wallet::wallet::Wallet;
use idp2p_wallet::wallet::WalletStore;
use std::convert::TryInto;

pub enum IdCommand {
    Create(String),
    SetDocument,
    Get(String),
    SendJwm { to: String, message: String },
}

impl IdCommand {
    pub fn handle(&self, behaviour: &mut IdentityGossipBehaviour) -> Result<()> {
        let store = FileStore {};
        match self {
            Self::Create(name) => {
                let seed = idp2p_common::create_random::<16>();
                let password = "123456";
                std::env::set_var("WALLET_SEED", idp2p_common::encode(&seed));
                std::env::set_var("WALLET_PASSWORD", password);
                std::env::set_var("ACCOUNT_NAME", name);
                let mut wallet = Wallet::new(password)?;
                let mut payload = wallet.resolve(password)?;
                let result = payload.create_account(name, seed)?;
                let id = result.account.did.id.clone();
                payload.accounts.push(result.account.clone());
                payload.next_index = result.next_index;
                let doc_result = payload.set_document(&name, seed.try_into().unwrap())?;
                payload.accounts[0] = doc_result.account.clone();
                payload.next_index = doc_result.next_index;
                wallet.save(password, payload.clone());
                store.put(&id, doc_result.account.did.clone());
                store.put_wallet(name, wallet);
                println!("Id: {}", id);
                behaviour.subscribe(id)?;
            }
            Self::SetDocument => {
                let seed = idp2p_common::decode(&std::env::var("WALLET_SEED")?);
                let password = "123456";
                let name = std::env::var("ACCOUNT_NAME")?;
                let mut wallet = store.get_wallet(&name).unwrap();
                let mut payload = wallet.resolve(password)?;
                let result = payload.set_document(&name, seed.try_into().unwrap())?;
                payload.accounts[0] = result.account.clone();
                payload.next_index = result.next_index;
                wallet.save(password, payload.clone());
                store.put(&result.account.did.id.clone(), result.account.did.clone());
                store.put_wallet(&name, wallet);
                let mes_payload = IdentityMessagePayload::Post {
                    digest: result.account.did.get_digest(),
                    identity: result.account.did.clone(),
                };
                let mes = IdentityMessage::new(mes_payload);
                behaviour.publish(result.account.did.id.clone(), mes)?;
            }
            Self::Get(id) => {
                behaviour.subscribe(id.clone())?;
                let mes = IdentityMessage::new(IdentityMessagePayload::Get);
                behaviour.publish(id.to_owned(), mes)?;
            }
            Self::SendJwm { to, message } => {
                let password = "123456";
                let name = std::env::var("ACCOUNT_NAME")?;
                let wallet = FileStore {}.get_wallet(&name).unwrap();
                let payload = wallet.resolve(password)?;
                let acc = payload.accounts[0].clone();
                let doc = acc.documents.unwrap()[0].clone();
                let to_did = store.get(&to).unwrap();
                let jwm = Jwm::new(acc.did.clone(), to_did, &message);
                let jwe = jwm.seal(EdSecret::from_bytes(&doc.authentication_secret))?;
                let json = serde_json::to_string(&jwe)?;
                let mes_payload = IdentityMessagePayload::Jwm { message: json };
                let mes = IdentityMessage::new(mes_payload);
                behaviour.publish(payload.accounts[0].did.id.clone(), mes)?;
            }
        }
        Ok(())
    }
}

pub fn get_command(input: &str) -> Option<IdCommand> {
    let split = input.split(" ");
    let input: Vec<&str> = split.collect();
    match input[0] {
        "create" => {
            // create alice
            return Some(IdCommand::Create(input[1].to_owned()));
        }
        "set-document" => {
            // set-document for alice
            return Some(IdCommand::SetDocument);
        }
        "get" => {
            // get <id>
            return Some(IdCommand::Get(input[1].to_owned()));
        }
        "send" => {
            // send <message> to <id>
            return Some(IdCommand::SendJwm {
                to: input[3].to_owned(),
                message: input[1].to_owned(),
            });
        }
        _ => {
            return None;
        }
    }
}

pub fn handle_jwm(jwm: &str) -> Result<()> {
    let store = FileStore {};
    let password = "123456";
    let name = std::env::var("ACCOUNT_NAME")?;
    let wallet = store.get_wallet(&name).unwrap();
    let payload = wallet.resolve(password)?;
    let acc = payload.accounts[0].clone();
    let doc = acc.documents.unwrap()[0].clone();
    let dec_secret = EdSecret::from_bytes(&doc.keyagreement_secret);
    let jwe: Jwe = serde_json::from_str(jwm)?;
    let json = jwe.decrypt(dec_secret)?;
    let jws: Jws = serde_json::from_str(&json)?;
    let jpm: Jpm = base64url::decode(&jws.payload)?;
    println!("Jpm: {:?}", jpm);
    let from = store.get(&jpm.from).unwrap();
    let jpm = jws.verify(from)?;
    println!("Received jwm {:?}", jpm);
    Ok(())
    // check  typ, enc, alg
    //let kid = self.recipients[0].header.kid.clone();
    //if kid != format!("{}")
    // check kid and secret
}
