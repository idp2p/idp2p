use crate::wallet::Wallet;
use idp2p_common::anyhow::Result;
use idp2p_core::did::Identity;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc::Sender;

pub enum WalletEvent {
    AccountCreated,
}

pub struct WalletResult {
    pub has_wallet: bool,
    pub has_session: bool,
}

pub struct WalletStore {
    pub wallet_path: PathBuf,
    shared: Arc<WalletShared>,
}

pub struct WalletShared {
    wallet: Mutex<Option<Wallet>>,
    event_sender: Sender<WalletEvent>,
}

impl WalletStore {
    pub fn new(wallet_path: PathBuf, event_sender: Sender<WalletEvent>) -> Result<Self> {
        let wallet = if wallet_path.is_file() {
            let mut file = File::open(wallet_path.as_path())?;
            let mut buff = String::new();
            file.read_to_string(&mut buff)?;
            let wallet = idp2p_common::serde_json::from_str::<Wallet>(&buff)?;
            Some(wallet)
        } else {
            None
        };
        let shared = Arc::new(WalletShared {
            wallet: Mutex::new(wallet),
            event_sender: event_sender,
        });
        Ok(WalletStore {
            wallet_path: wallet_path,
            shared: shared,
        })
    }

    pub async fn register(&self, password: &str) -> Result<()> {
        let _ = self.shared
            .event_sender
            .send(WalletEvent::AccountCreated)
            .await;
        Ok(())
    }

    pub fn login(&self, password: &str) {}

    pub fn logout(&self, password: &str) {}

    pub fn get_state(&self) -> WalletResult {
        let wallet = self.shared.wallet.lock().unwrap();
        if wallet.is_none() {}
        WalletResult {
            has_wallet: false,
            has_session: false,
        }
    }
}

