use crate::raw::{Connection, RawWallet, SharedWallet};
use crate::session::{self, SecretWallet, SessionState, WalletSession};
use crate::wallet::Wallet;
use crate::{derive_secret, get_enc_key, WalletPersister};
use chacha20poly1305::aead::{Aead, NewAead};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use idp2p_common::anyhow::Result;
use idp2p_common::{log, serde_json};
use idp2p_core::did::Identity;
use idp2p_core::message::IdentityMessage;
use idp2p_didcomm::jpm::Jpm;
use idp2p_didcomm::jwm::{JwmBody, IdProfile};
use idp2p_didcomm::jws::Jws;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct WalletState {
    #[serde(flatten)]
    pub raw: RawWallet,
    pub session: Option<SessionState>,
}

pub struct WalletStore<T: WalletPersister> {
    pub wallet: Mutex<Wallet<T>>,
}

impl<T> WalletStore<T>
where
    T: WalletPersister,
{
    pub fn new(persister: T) -> Self {
        let mut wallet = Wallet {
            persister: persister,
            raw: None,
            session: None,
        };
        if wallet.persister.wallet_exists() {
            log::info!("Persisted wallet is importing");
            wallet.raw = Some(wallet.persister.get_wallet().unwrap().raw);
        }
        WalletStore {
            wallet: Mutex::new(wallet),
        }
    }

    pub fn get_state(&self) -> Option<WalletState> {
        let wallet = self.wallet.lock().unwrap();
        if let Some(ref raw) = wallet.raw {
            if let Some(ref session) = wallet.session {
                return Some(WalletState {
                    raw: raw.clone(),
                    session: Some(session.to_state()),
                });
            }
            return Some(WalletState {
                raw: raw.clone(),
                session: None,
            });
        }
        None
    }

    pub fn register(&self, name: &str, photo: &[u8], pwd: &str) -> Result<(Identity, [u8; 16])> {
        let mut wallet = self.wallet.lock().unwrap();
        if wallet.raw.is_none() {
            let seed = idp2p_common::create_random::<16>();
            let mut next_index = 1000000000;
            let secret = derive_secret(seed, &mut next_index)?;
            let did = Identity::from_secret(secret.clone());
            let shared = SharedWallet{
                next_index: next_index,
                next_secret_index: 1000000000,
                recovery_secret_index: 1000000000,
                assertion_secret_index: 1000000000,
                authentication_secret_index: 1000000000,
                agreement_secret_index: 1000000000,
            };
            wallet.raw = Some(RawWallet::new(name, photo, did.id.as_str(), shared)?);
            wallet.session = Some(WalletSession::new_with_secret(secret, pwd));
            wallet.persist()?;
            return Ok((did, seed));
        }
        idp2p_common::anyhow::bail!("Identity already exists")
    }

    pub fn login(&self, password: &str) -> Result<()> {
        let mut wallet = self.wallet.lock().unwrap();
        let raw = wallet.raw.clone().unwrap();
        let persisted_wallet = wallet.persister.get_wallet()?;
        let enc_key_bytes = get_enc_key(password, &raw.salt).unwrap();
        let enc_key = Key::from_slice(&enc_key_bytes);
        let cipher = ChaCha20Poly1305::new(enc_key);
        let nonce = Nonce::from_slice(&raw.iv);
        let result = cipher
            .decrypt(nonce, persisted_wallet.ciphertext.as_ref())
            .unwrap();

        let secret_wallet: SecretWallet = serde_json::from_slice(&result)?;
        wallet.session = Some(WalletSession::new(secret_wallet, password));
        Ok(())
    }

    pub fn logout(&self) {
        let mut wallet = self.wallet.lock().unwrap();
        wallet.session = None;
    }

    pub fn get_agreement_secret(&self) -> Vec<u8>{
        let wallet = self.wallet.lock().unwrap();
        if let Some(session) = wallet.session.clone() {
            return session.secret.keyagreement_secret;
        }
        vec![]
    }
    
    pub fn connect(&self, to: Identity) -> Result<IdentityMessage> {
        let mut wallet = self.wallet.lock().unwrap();
        if let Some(session) = wallet.session.clone() {
            if let Some(ref mut raw) = wallet.raw {
                raw.add_request(&to.id);
                let body = JwmBody::Accept(IdProfile::new(&raw.name, &raw.photo));
                
                /*let jwm = raw.identity.new_jwm(to.clone(), body);
                let jwm_str = session.create_jwm(jwm)?;
                let id_mes = IdentityMessage::new_jwm(&to.id, &jwm_str);*/
                wallet.persist()?;
                //return Ok(id_mes);
            }
        }
        idp2p_common::anyhow::bail!("Session not found");
    }

    pub fn accept(&self, to: Identity) -> Result<IdentityMessage> {
        let mut wallet = self.wallet.lock().unwrap();
        if let Some(session) = wallet.session.clone() {
            if let Some(ref mut raw) = wallet.raw {
                /*let id = to.id.clone();
                raw.accept_conn(&id);
                let body = JwmBody::Connect(raw.profile.clone());
                let jwm = raw.identity.new_jwm(to.clone(), body);
                let jwm_str = session.create_jwm(jwm)?;
                let id_mes = IdentityMessage::new_jwm(&id, &jwm_str);
                wallet.persist()?;
                return Ok(id_mes);*/
            }
        }
        idp2p_common::anyhow::bail!("Session not found");
    }

    pub fn send_msg(&self, to: Identity, msg: &str) -> Result<IdentityMessage> {
        let mut wallet = self.wallet.lock().unwrap();
        if let Some(session) = wallet.session.clone() {
            if let Some(ref mut raw) = wallet.raw {
                /*let id = to.id.clone();
                let body = JwmBody::Message(msg.to_owned());
                let jwm = raw.identity.new_jwm(to.clone(), body);
                let jwm_str = session.create_jwm(jwm)?;
                let id_mes = IdentityMessage::new_jwm(&id, &jwm_str);
                wallet.persist()?;
                return Ok(id_mes);*/
            }
        }
        idp2p_common::anyhow::bail!("Session not found");
    }

    pub fn handle_jwm(&self, jpm: Jpm) -> Result<()> {
        let mut wallet = self.wallet.lock().unwrap();
        if let Some(ref mut raw) = wallet.raw {
            /*match body {
                JwmBody::Connect(profile) => {
                    raw.add_conn(Connection::new(id, profile));
                }
                JwmBody::Accept(profile) => {
                    raw.add_conn(Connection::new(id, profile));
                    raw.accept_conn(id);
                    raw.remove_request(id);
                }
                JwmBody::Message(msg) => {
                    raw.add_received_message(id, &msg);
                }
                _=>{}
            }*/
            wallet.persist()?;
            return Ok(());
        }
        idp2p_common::anyhow::bail!("Session not found");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    /*#[test]
    fn register_test() -> Result<()> {
        let store = WalletStore::new(MockPersister::new());
        let profile = IdProfile::new("Adem", &vec![]);
        store.register(profile, "123456")?;
        let state = store.get_state()?;
        assert!(state.exists);
        assert!(state.session.is_some());
        assert_eq!(state.session.unwrap().raw_wallet.profile.name, "Adem");
        Ok(())
    }

    #[test]
    fn login_test() -> Result<()> {
        let store = WalletStore::new(MockPersister::new());
        let profile = IdProfile::new("Adem", &vec![]);
        store.register(profile, "123456")?;
        store.logout();
        store.login("123456")?;
        let state = store.get_state()?;
        assert!(state.exists);
        assert!(state.session.is_some());
        Ok(())
    }

    struct MockPersister {
        wallet: RefCell<Vec<String>>,
    }

    impl MockPersister {
        fn new() -> Self {
            Self {
                wallet: RefCell::new(vec![]),
            }
        }
    }

    impl WalletPersister for MockPersister {
        fn wallet_exists(&self) -> bool {
            !self.wallet.borrow().is_empty()
        }
        fn get_wallet(&self) -> Result<String> {
            let s = self.wallet.borrow_mut();
            Ok(s[0].clone())
        }
        fn persist_wallet(&self, enc_wallet: &str) -> Result<()> {
            let mut w = self.wallet.borrow_mut();
            w.push(enc_wallet.to_owned());
            Ok(())
        }
    }*/
}
