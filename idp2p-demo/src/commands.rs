use idp2p_wallet::wallet::WalletStore;
use crate::FileStore;
use idp2p_common::anyhow::*;
use idp2p_node::behaviour::IdentityGossipBehaviour;
use idp2p_node::message::{IdentityMessage, IdentityMessagePayload};
use idp2p_node::store::IdStore;
use idp2p_wallet::wallet::Wallet;
use std::convert::TryInto;

pub enum IdCommand {
    Create(String),
    SetDocument,
    Get(String),
    SendJwm {
        to: String,
        message: String,
    },
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
                payload.accounts.push(result.account.clone()); 
                payload.next_index = result.next_index;
                wallet.save(password, payload.clone());
                store.put(&result.account.did.id.clone(), result.account.did.clone());
                store.put_wallet(name, wallet);
                println!("Id: {}", result.account.did.id);
                behaviour.subscribe(result.account.did.id)?;
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
                let mes_payload = IdentityMessagePayload::Post{
                    digest:result.account.did.get_digest(),
                    identity: result.account.did.clone() 
                };
                let mes = IdentityMessage::new(mes_payload);
                behaviour.publish(result.account.did.id.clone(), mes)?;   
            }
            Self::Get(id) => {
                behaviour.subscribe(id.clone())?;
                let mes = IdentityMessage::new(IdentityMessagePayload::Get);
                behaviour.publish(id.to_owned(), mes)?;
            }
            Self::SendJwm { to, message } => {}
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
