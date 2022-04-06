use crate::file_db::FilePersister;
use idp2p_common::anyhow::Result;
use idp2p_core::{store::IdStore, IdProfile, IdentityEvent};
use idp2p_wallet::store::{WalletState, WalletStore};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum IdCommand {
    Get,
    Register {
        profile: IdProfile,
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

type Ws = Arc<WalletStore<FilePersister>>;
type Tx = Sender<IdentityEvent>;

impl IdCommand {
    pub async fn handle(
        &self,
        ws: Ws,
        id_store: Arc<IdStore>,
        tx: Tx,
    ) -> Result<Option<WalletState>> {
        match &self {
            Self::Register { profile, password } => {
                ws.register(profile.clone(), password)?;
                let did = ws.get_state().unwrap().raw.identity.clone();
                id_store.create_did(did).await;
            }
            Self::Login { password } => {
                ws.login(password)?;
            }
            Self::Connect { id } => {
                let event = IdentityEvent::Connected { id: id.to_owned() };
                tx.send(event).await?;
                //let to = id_store.get_did(id);

                //id_store.
                //let message = ws.connect(to)?;
            }
            _ => {}
        }
        Ok(ws.get_state())
    }
}
