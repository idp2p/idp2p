use crate::enc_wallet::Wallet;
use std::sync::Mutex;
use crate::raw_wallet::RawWallet;
use idp2p_node::store::IdShared;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use idp2p_common::anyhow::Result;

pub enum WalletEvent {
    MessageReceived,
}

pub struct WalletOptions {
    pub wallet_path: PathBuf,
    pub event_sender: Sender<WalletEvent>,
    pub id_shared: Arc<IdShared>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct WalletState {
    pub is_exist: bool,
    pub session_crat: Option<i64>,
    pub session_exp: Option<i64>,
    pub session_wallet: Option<RawWallet>,
}

pub struct WalletShared {
    pub wallet: Mutex<Wallet>,
    pub event_sender: Sender<WalletEvent>,
}

#[derive(Clone)]
pub struct WalletStore {
    pub path: PathBuf,
    pub wallet_shared: Arc<WalletShared>,
    pub id_shared: Arc<IdShared>,
}

impl WalletStore {
    pub fn new(options: WalletOptions) -> Result<Self> {
        let wallet = Wallet{
           path: options.wallet_path.clone(),
           session: None
        };
        let shared = Arc::new(WalletShared {
            wallet: Mutex::new(wallet),
            event_sender: options.event_sender,
        });
        Ok(WalletStore {
            path: options.wallet_path,
            wallet_shared: shared,
            id_shared: options.id_shared,
        })
    }

    pub fn get_state(&self) -> Result<WalletState> {
        let wallet = self.wallet_shared.wallet.lock().unwrap();
        Ok(WalletState{
           is_exist: true,
           session_crat: None,
           session_exp: None,
           session_wallet: None
        })
    }

    pub async fn register(&self, username: &str, password: &str) -> Result<Vec<u8>> {
        if !std::path::Path::new(&self.path).exists() {
            std::fs::File::create(&self.path)?;
        }
        Ok(vec![])
    }

    pub async fn login(&self, password: &str) -> Result<()> {
        let mut wallet = self.wallet_shared.wallet.lock().unwrap();
        
        Ok(())
    }

    pub async fn logout(&self) {
        let mut wallet = self.wallet_shared.wallet.lock().unwrap();
        wallet.session = None;
    }

    pub async fn connect(&self, id: &str, username: &str) -> Result<()> {
        let mut wallet = self.wallet_shared.wallet.lock().unwrap();
        if let Some(ref mut session) = wallet.session {
            session.raw.add_conn(id, username);
            wallet.persist()?;
        }
        Ok(())
    }

    pub async fn send_message(&self, to: &str, message: &str) -> Result<()> {
        let mut wallet = self.wallet_shared.wallet.lock().unwrap();
        if let Some(ref mut session) = wallet.session {
            session.send_message(to, message)?;
            wallet.persist()?;
        }
        Ok(())
    }

    pub async fn handle_jwm(&self, jwm: &str) -> Result<()> {
        let mut wallet = self.wallet_shared.wallet.lock().unwrap();
        if let Some(ref mut session) = wallet.session {
            session.handle_jwm(jwm)?;
            wallet.persist()?;
        }
        Ok(())
    }
}

async fn _listen_session_ttl() {
    // to do(remove session)
}