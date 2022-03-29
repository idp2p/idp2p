use crate::raw::RawWallet;
use crate::wallet::Wallet;
use idp2p_common::anyhow::Result;
use idp2p_core::did::Identity;
use idp2p_core::store::IdStore;
use idp2p_didcomm::jwm::JwmBody;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

pub struct WalletOptions {
    pub wallet_path: PathBuf
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct WalletState {
    pub is_exist: bool,
    pub session_crat: Option<i64>,
    pub session_exp: Option<i64>,
    pub session_wallet: Option<RawWallet>,
}

pub struct WalletStore {
    pub wallet: Mutex<Wallet>
}

impl WalletStore {
    pub fn new(options: WalletOptions) -> Result<Self> {
        let wallet = Wallet {
            path: options.wallet_path.clone(),
            session: None,
        };
        let store = WalletStore {
            wallet: Mutex::new(wallet)
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

    pub async fn register(&self, username: &str, password: &str) -> Result<(Identity, [u8;16])> {
        let mut wallet = self.wallet.lock().unwrap();
        std::fs::File::create(&wallet.path)?;
        let (did, seed) = wallet.register(username, password)?;
        wallet.persist()?;
        Ok((did, seed))
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

    pub fn connect(&self, id: &str) -> Result<bool> {
        let mut wallet = self.wallet.lock().unwrap();
        if let Some(ref mut session) = wallet.session {
            session.raw.add_request(id);
            wallet.persist()?;
            return Ok(true);
        }
        idp2p_common::anyhow::bail!("Session not found");    
    }

    pub async fn accept(&self, id: &str) -> Result<bool> {
        let mut wallet = self.wallet.lock().unwrap();
        if let Some(ref mut session) = wallet.session {
            //session.raw.add_conn(id);
            wallet.persist()?;
        }
        idp2p_common::anyhow::bail!("Session not found"); 
    }

    pub fn send_message(&self, id_store: Arc<IdStore>, to: &str, message: &str) -> Result<()> {
        let mut wallet = self.wallet.lock().unwrap();
        if let Some(ref mut session) = wallet.session {
            session.send_jwm(id_store, to, JwmBody::Message(message.to_owned()))?;
            wallet.persist()?;
        }
        idp2p_common::anyhow::bail!("Session not found"); 
    }

    pub async fn handle_jwm(&self, id_store: Arc<IdStore>, jwm: &str) -> Result<()> {
        let mut wallet = self.wallet.lock().unwrap();
        if let Some(ref mut session) = wallet.session {
            session.handle_jwm(id_store, jwm)?;
            wallet.persist()?;
        }
        idp2p_common::anyhow::bail!("Session not found"); 
    }
}

async fn _listen_session_ttl() {
    // to do(remove session)
}
