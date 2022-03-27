use idp2p_core::did::Identity;
use crate::raw::RawWallet;
use crate::wallet::Wallet;
use idp2p_common::anyhow::Result;
use idp2p_core::store::IdStore;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc::Sender;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum WalletEvent {
    Created(Identity),
    Connected(String),
    MessageReceived(String),
}

pub struct WalletOptions {
    pub wallet_path: PathBuf,
    pub event_sender: Sender<WalletEvent>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct WalletState {
    pub is_exist: bool,
    pub session_crat: Option<i64>,
    pub session_exp: Option<i64>,
    pub session_wallet: Option<RawWallet>,
}

pub struct WalletStore {
    pub wallet: Mutex<Wallet>,
    pub event_sender: Sender<WalletEvent>,
}

impl WalletStore {
    pub fn new(options: WalletOptions) -> Result<Self> {
        let wallet = Wallet {
            path: options.wallet_path.clone(),
            session: None,
        };
        let store =WalletStore {
            wallet: Mutex::new(wallet),
            event_sender: options.event_sender,
        };
        Ok(store)
    }

    pub fn get_state(&self) -> Result<WalletState> {
        let wallet = self.wallet.lock().unwrap();
        let mut state = WalletState {
            is_exist: true,
            session_crat: None,
            session_exp: None,
            session_wallet: None,
        };
        if std::path::Path::new(&wallet.path).exists() {
            state.is_exist = true;
            if let Some(ref session) = wallet.session {
                state.session_crat = Some(session.created_at);
                state.session_exp = Some(session.expire_at);
                state.session_wallet = Some(session.raw.clone());
            }
        }

        Ok(state)
    }

    pub async fn register(&self, username: &str, password: &str) -> Result<Vec<u8>> {
        let mut wallet = self.wallet.lock().unwrap();
        std::fs::File::create(&wallet.path)?;
        let (did, _) = wallet.register(username, password)?;
        wallet.persist()?;
        let event = WalletEvent::Created(did);
        self.event_sender.send(event).await.unwrap();
        Ok(vec![])
    }

    pub async fn login(&self, password: &str) -> Result<()> {
        let mut wallet = self.wallet.lock().unwrap();
        wallet.login(password)?;
        Ok(())
    }

    pub async fn logout(&self) {
        let mut wallet = self.wallet.lock().unwrap();
        wallet.session = None;
    }

    pub async fn connect(&self, id: &str, username: &str) -> Result<()> {
        let mut wallet = self.wallet.lock().unwrap();
        if let Some(ref mut session) = wallet.session {
            session.raw.add_conn(id, username);
            wallet.persist()?;
        }
        let event = WalletEvent::Connected(id.to_owned());
        self.event_sender.send(event).await.unwrap();
        Ok(())
    }

    pub async fn send_message(&self, id_store: Arc<IdStore>, to: &str, message: &str) -> Result<()> {
        let mut wallet = self.wallet.lock().unwrap();
        if let Some(ref mut session) = wallet.session {
            session.send_message(id_store, to, message)?;
            wallet.persist()?;
        }
        Ok(())
    }

    pub async fn handle_jwm(&self, id_store: Arc<IdStore>, jwm: &str) -> Result<()> {
        let mut wallet = self.wallet.lock().unwrap();
        if let Some(ref mut session) = wallet.session {
            session.handle_jwm(id_store, jwm)?;
            wallet.persist()?;
        }
        Ok(())
    }
}

async fn _listen_session_ttl() {
    // to do(remove session)
}
