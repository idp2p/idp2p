use crate::wallet::Wallet;
use idp2p_common::anyhow::Result;
use idp2p_node::IdentityEvent;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc::Sender;

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
    event_sender: Sender<IdentityEvent>,
}

impl WalletStore {
    pub fn new(wallet_path: PathBuf, event_sender: Sender<IdentityEvent>) -> Result<Self> {
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
        tokio::spawn(listen_changes(shared.clone()));
        Ok(WalletStore {
            wallet_path: wallet_path,
            shared: shared,
        })
    }

    pub async fn create_wallet(&self) {
        self.shared
            .event_sender
            .send(IdentityEvent::Published {
                id: "id".to_owned(),
            })
            .await
            .unwrap();
    }

    pub fn create_session(&self) {}

    pub fn remove_session(&self) {}

    pub fn get_state(&self) -> WalletResult {
        let wallet = self.shared.wallet.lock().unwrap();
        if wallet.is_none() {}
        WalletResult {
            has_wallet: false,
            has_session: false,
        }
    }
}

async fn listen_changes(shared: Arc<WalletShared>) {}
