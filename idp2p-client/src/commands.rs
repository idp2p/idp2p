use crate::file_db::FilePersister;
use idp2p_common::ed_secret::EdSecret;
use idp2p_common::{anyhow::Result, serde_json};
use idp2p_common::encode_vec;
use idp2p_core::{
    message::{IdentityMessage, IdentityMessagePayload},
    store::IdStore,
    IdentityEvent,
};
use idp2p_didcomm::jpm::Jpm;
use idp2p_didcomm::jwe::Jwe;
use idp2p_didcomm::jws::Jws;
use idp2p_wallet::{
    store::{WalletState, WalletStore},
    WalletPersister,
};
use libp2p::gossipsub::GossipsubEvent;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

pub struct IdCommandHandler<T: WalletPersister> {
    pub wallet_store: Arc<WalletStore<T>>,
    pub id_store: Arc<IdStore>,
    pub event_sender: Sender<IdentityEvent>,
}

impl<T> IdCommandHandler<T>
where
    T: WalletPersister,
{
    pub async fn handle_gossip_event(&self, event: GossipsubEvent) -> Result<()> {
        if let GossipsubEvent::Message {
            propagation_source: _,
            message_id: _,
            message,
        } = event
        {
            let topic = message.topic.to_string();
            if idp2p_common::is_idp2p(&topic) {
                let message = IdentityMessage::from_bytes(&message.data);
                match &message.payload {
                    IdentityMessagePayload::Get => {
                        self.id_store.handle_get(&topic).await;
                    }
                    IdentityMessagePayload::Post { digest, identity } => {
                        self.id_store.handle_post(digest, identity).await.unwrap();
                    }
                    IdentityMessagePayload::Jwm { jwm } => {
                        let secret = self.wallet_store.get_agreement_secret();
                        let jwe: Jwe = serde_json::from_str(jwm)?;
                        /*let expected_kid = self.wallet_store.wallet.
                        if doc.get_first_keyagreement() != jwe.recipients[0].header.kid {
                            idp2p_common::anyhow::bail!("INVALID_KID");
                        }*/
                        let dec_secret = EdSecret::from_bytes(&secret);
                        let json = jwe.decrypt(dec_secret)?;
                        let jws: Jws = serde_json::from_str(&json)?;
                        let jpm = Jpm::from_str(&jws.payload)?;
                        jws.verify(&[0u8; 32])?;
                        // decrypt
                        // send event with raw data
                    }
                }
            }
        }
        Ok(())
    }
    
    pub async fn handle(&self, cmd: IdCommand) -> Result<Option<WalletState>> {
        match &cmd {
            IdCommand::Register {
                name,
                photo,
                password,
            } => {
                let (did, _) = self.wallet_store.register(name, photo, password)?;
                self.id_store.create_did(did).await;
            }
            IdCommand::Login { password } => {
                self.wallet_store.login(password)?;
            }
            IdCommand::Connect { id } => {
                //let event = IdentityEvent::Connected { id: id.to_owned() };
                //self.event_sender.send(event).await?;
                //let to = id_store.get_did(id);

                //id_store.
                //let message = ws.connect(to)?;
            }
            _ => {}
        }
        Ok(self.wallet_store.get_state())
    }
}

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum IdCommand {
    Get,
    Register {
        name: String,
        #[serde(with = "encode_vec")]
        photo: Vec<u8>,
        password: String,
    },
    Login {
        password: String,
    },
    Connect {
        id: String,
    },
    Accept(String),
    SendMessage {
        id: String,
        msg: String,
    },
}
