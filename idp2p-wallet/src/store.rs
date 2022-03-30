use crate::Persister;
use crate::raw::RawWallet;
use crate::wallet::Wallet;
use idp2p_common::anyhow::Result;
use idp2p_core::did::Identity;
use idp2p_core::message::IdentityMessage;
use idp2p_didcomm::jwm::JwmBody;
use idp2p_didcomm::jws::Jws;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct WalletState {
    pub is_exist: bool,
    pub session_crat: Option<i64>,
    pub session_exp: Option<i64>,
    pub session_wallet: Option<RawWallet>,
}

pub struct WalletStore<T: Persister> {
    pub wallet: Mutex<Wallet<T>>,
}

impl <T> WalletStore<T> where T: Persister {
    pub fn new(persister: T) -> Result<Self> {
        let wallet = Wallet {
            persister: persister,
            session: None,
        };
        let store = WalletStore {
            wallet: Mutex::new(wallet),
        };
        Ok(store)
    }

    pub fn get_state(&self) -> Result<WalletState> {
        let wallet = self.wallet.lock().unwrap();
        let mut state = WalletState {
            is_exist: false,
            session_crat: None,
            session_exp: None,
            session_wallet: None,
        };
        if wallet.persister.exists() {
            state.is_exist = true;
            if let Some(ref session) = wallet.session {
                state.session_crat = Some(session.created_at);
                state.session_exp = Some(session.expire_at);
                state.session_wallet = Some(session.raw.clone());
            }
        }

        Ok(state)
    }

    pub fn register(&self, name: &str, password: &str, photo: &[u8]) -> Result<(Identity, [u8; 16])> {
        let mut wallet = self.wallet.lock().unwrap();
        let (did, seed) = wallet.register(name, password, photo)?;
        wallet.persist()?;
        Ok((did, seed))
    }

    pub fn login(&self, password: &str) -> Result<()> {
        let mut wallet = self.wallet.lock().unwrap();
        wallet.login(password)?;
        Ok(())
    }

    pub fn logout(&self) {
        let mut wallet = self.wallet.lock().unwrap();
        wallet.session = None;
    }

    pub fn connect(&self, id: &str) -> Result<IdentityMessage> {
        let mut wallet = self.wallet.lock().unwrap();
        if let Some(ref mut session) = wallet.session {
            session.raw.add_request(id);
            wallet.persist()?;
        }
        idp2p_common::anyhow::bail!("Session not found");
    }

    pub fn accept(&self, id: &str) -> Result<IdentityMessage> {
        let mut wallet = self.wallet.lock().unwrap();
        if let Some(ref mut session) = wallet.session {
            session.raw.accept_conn(id);
            wallet.persist()?;
        }
        idp2p_common::anyhow::bail!("Session not found");
    }

    pub fn send_msg(&self, to: Identity, msg: &str) -> Result<IdentityMessage> {
        let mut wallet = self.wallet.lock().unwrap();
        if let Some(ref mut session) = wallet.session {
            let id = to.id.clone();
            let body = JwmBody::Message(msg.to_owned());
            let jwm_str = session.create_jwm(to, body)?;
            let id_mes = IdentityMessage::new_jwm(&id, &jwm_str);
            wallet.persist()?;
            return Ok(id_mes);
        }
        idp2p_common::anyhow::bail!("Session not found");
    }

    pub fn resolve_jwe(&self, jwe: &str) -> Result<Jws> {
        let mut wallet = self.wallet.lock().unwrap();
        if let Some(ref mut session) = wallet.session {
            let jws = session.resolve_jwe(jwe)?;
            return Ok(jws);
        }
        idp2p_common::anyhow::bail!("Session not found");
    }

    pub fn handle_jwm(&self, body: JwmBody) -> Result<()> {
        let mut wallet = self.wallet.lock().unwrap();
        if let Some(ref mut session) = wallet.session {
            match body {
                JwmBody::Connect(req) => {

                }
                JwmBody::Message(msg) => {}
            }
            wallet.persist()?;
            return Ok(());
        }
        idp2p_common::anyhow::bail!("Session not found");
    }
}

