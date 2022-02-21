use crate::FileStore;
use idp2p_common::anyhow::*;
use idp2p_node::behaviour::IdentityGossipBehaviour;
use idp2p_node::message::{IdentityMessage, IdentityMessagePayload};
use idp2p_node::store::IdStore;
use idp2p_wallet::wallet::Wallet;

pub enum IdCommand {
    Create(String),
    SetDocument(String),
    Get(String),
    SendJwm {
        from: String,
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
                let wallet = Wallet::new(password)?;
                let payload = wallet.get_payload(password)?;
                let result = payload.create_account(name, seed)?;
                store.put(&result.did.id.clone(), result.did.clone());
                behaviour.subscribe(result.did.id)?;
            }
            Self::SetDocument(name) => {}
            Self::Get(id) => {
                behaviour.subscribe(id.clone())?;
                let mes_payload = IdentityMessagePayload::Get;
                let mes = IdentityMessage::new(mes_payload);
                behaviour.publish(id.to_owned(), mes)?;
            }
            Self::SendJwm { from, to, message } => {}
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
            return Some(IdCommand::SetDocument(input[2].to_owned()));
        }
        "get" => {
            // get <id>
            return Some(IdCommand::Get(input[1].to_owned()));
        }
        "send-message" => {
            // send-message from <name> to <id> <message>
            return Some(IdCommand::SendJwm {
                from: input[2].to_owned(),
                to: input[4].to_owned(),
                message: input[5].to_owned(),
            });
        }
        _ => {
            return None;
        }
    }
}

/*

"create" => {
            let name = input[1];
            let base_path = format!("../target/{}", name);
            std::env::set_var("BASE_PATH", base_path.clone());
            let id_path = format!("{}/identities", base_path);
            std::fs::create_dir_all(id_path).unwrap();
            let acc_path = format!("{}/accounts", base_path);
            std::fs::create_dir_all(acc_path).unwrap();
            let seed = idp2p_common::create_random::<16>();
            let password = "123456";
            let wallet = Wallet::new(password)?;
            let payload = wallet.get_payload(password)?;
            let result = payload.create_account(name, seed)?;
        }
        "set-document" => {
            let name = input[1];
        }
        "get" => {
            let id = input[1].to_string();
            behaviour.subscribe(id.clone())?;
            let mes_payload = IdentityMessagePayload::Get;
            let mes = IdentityMessage::new(mes_payload);
            behaviour.publish(id, mes)?;
        }
        "send-message" => {
            let message_data = input[1].to_owned();
            let receiver_id = input[2].to_owned();
            let mes_payload = IdentityMessagePayload::Jwm {
                message: message_data,
            };
            let mes = IdentityMessage::new(mes_payload);

            behaviour.publish(receiver_id, mes)?;
        }
        _ => {
            println!("Unknown command");
        }
*/
