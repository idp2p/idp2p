use crate::raw::{Connection, RawWallet, IdConfig};
use crate::secret::SecretWallet;
use crate::session::WalletSession;
use crate::wallet::{EncryptedWallet, EncryptedWalletPayload, Wallet};
use crate::{derive_secret, get_enc_key, Persister};
use chacha20poly1305::aead::{Aead, NewAead};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use idp2p_common::{anyhow::Result, serde_json};
use idp2p_core::did::Identity;
use idp2p_core::message::IdentityMessage;
use idp2p_core::IdProfile;
use idp2p_didcomm::jwm::JwmBody;
use idp2p_didcomm::jws::Jws;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct WalletState {
    pub exists: bool,
    pub session_crat: Option<i64>,
    pub session_exp: Option<i64>,
    pub session_wallet: Option<RawWallet>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct CreateWalletInput{
    pub profile: IdProfile,
    pub config: IdConfig,
    pub password: String
}
pub struct WalletStore<T: Persister> {
    pub wallet: Mutex<Wallet<T>>,
}

impl<T> WalletStore<T>
where
    T: Persister,
{
    pub fn new(persister: T) -> Self {
        let wallet = Wallet {
            persister: persister,
            session: None,
        };
        WalletStore {
            wallet: Mutex::new(wallet),
        }
    }

    pub fn get_state(&self) -> Result<WalletState> {
        let wallet = self.wallet.lock().unwrap();
        let mut state = WalletState {
            exists: false,
            session_crat: None,
            session_exp: None,
            session_wallet: None,
        };
        if wallet.persister.exists() {
            state.exists = true;
            if let Some(ref session) = wallet.session {
                state.session_crat = Some(session.created_at);
                state.session_exp = Some(session.expire_at);
                state.session_wallet = Some(session.raw.clone());
            }
        }

        Ok(state)
    }

    pub fn register(&self, input: CreateWalletInput) -> Result<(Identity, [u8; 16])> {
        let mut wallet = self.wallet.lock().unwrap();
        let seed = idp2p_common::create_random::<16>();
        let iv = idp2p_common::create_random::<12>();
        let salt = idp2p_common::create_random::<16>();
        let mut next_index = 1000000000;
        let secret = derive_secret(seed, &mut next_index)?;
        let did = Identity::from_secret(secret.clone());
        let raw_wallet = RawWallet::new(input.profile, input.config, did.clone());
        let secret_wallet = SecretWallet {
            next_index: next_index,
            next_secret_index: next_index,
            recovery_secret_index: next_index,
            assertion_secret: secret.to_bytes().to_vec(),
            authentication_secret: secret.to_bytes().to_vec(),
            keyagreement_secret: secret.to_bytes().to_vec(),
        };

        wallet.session = Some(WalletSession {
            raw: raw_wallet,
            secret: secret_wallet,
            created_at: 0,
            expire_at: 0,
            password: input.password,
            salt: salt,
            iv: iv,
        });
        wallet.persist()?;
        Ok((did, seed))
    }

    pub fn login(&self, password: &str) -> Result<()> {
        let mut wallet = self.wallet.lock().unwrap();
        let wallet_str = wallet.persister.get()?;
        let enc_wallet = serde_json::from_str::<EncryptedWallet>(&wallet_str)?;
        let enc_key_bytes = get_enc_key(password, &enc_wallet.salt).unwrap();
        let enc_key = Key::from_slice(&enc_key_bytes);
        let cipher = ChaCha20Poly1305::new(enc_key);
        let nonce = Nonce::from_slice(&enc_wallet.iv);
        let result = cipher
            .decrypt(nonce, enc_wallet.ciphertext.as_ref())
            .unwrap();
        let payload: EncryptedWalletPayload = serde_json::from_slice(&result)?;
        wallet.session = Some(WalletSession {
            raw: payload.raw,
            secret: payload.secret,
            created_at: 0,
            expire_at: 0,
            password: password.to_owned(),
            salt: enc_wallet.salt.try_into().unwrap(),
            iv: enc_wallet.iv.try_into().unwrap(),
        });
        Ok(())
    }

    pub fn logout(&self) {
        let mut wallet = self.wallet.lock().unwrap();
        wallet.session = None;
    }

    pub fn connect(&self, to: Identity) -> Result<IdentityMessage> {
        let mut wallet = self.wallet.lock().unwrap();
        if let Some(ref mut session) = wallet.session {
            let id = to.id.clone();
            session.raw.add_request(&id);
            let body = JwmBody::Accept(session.raw.profile.clone());
            let jwm_str = session.create_jwm(to, body)?;
            let id_mes = IdentityMessage::new_jwm(&id, &jwm_str);
            wallet.persist()?;
            return Ok(id_mes);
        }
        idp2p_common::anyhow::bail!("Session not found");
    }

    pub fn accept(&self, to: Identity) -> Result<IdentityMessage> {
        let mut wallet = self.wallet.lock().unwrap();
        if let Some(ref mut session) = wallet.session {
            let id = to.id.clone();
            session.raw.accept_conn(&id);
            let body = JwmBody::Connect(session.raw.profile.clone());
            let jwm_str = session.create_jwm(to, body)?;
            let id_mes = IdentityMessage::new_jwm(&id, &jwm_str);
            wallet.persist()?;
            return Ok(id_mes);
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

    pub fn handle_jwm(&self, id: &str, body: JwmBody) -> Result<()> {
        let mut wallet = self.wallet.lock().unwrap();
        if let Some(ref mut session) = wallet.session {
            match body {
                JwmBody::Connect(profile) => {
                    session.raw.add_conn(Connection::new(id, profile));
                }
                JwmBody::Accept(profile) => {
                    session.raw.add_conn(Connection::new(id, profile));
                    session.raw.accept_conn(id);
                    session.raw.remove_request(id);
                }
                JwmBody::Message(msg) => {
                    session.raw.add_received_message(id, &msg);
                }
            }
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
    #[test]
    fn register_test() -> Result<()> {
        let store = WalletStore::new(MockPersister::new());
        let profile = IdProfile::new("Adem", &vec![]);
        let config = IdConfig::new(&vec![], 43727);
        let input = CreateWalletInput{
            config: config,
            profile: profile,
            password: "123456".to_owned()
        };
        store.register(input)?;
        let state = store.get_state()?;
        assert!(state.exists);
        assert!(state.session_wallet.is_some());
        assert_eq!(state.session_wallet.unwrap().profile.name, "Adem");
        Ok(())
    }

    #[test]
    fn login_test() -> Result<()> {
        let store = WalletStore::new(MockPersister::new());
        let profile = IdProfile::new("Adem", &vec![]);
        let config = IdConfig::new(&vec![], 43727);
        let input = CreateWalletInput{
            config: config,
            profile: profile,
            password: "123456".to_owned()
        };
        store.register(input)?;
        store.logout();
        store.login("123456")?;
        let state = store.get_state()?;
        assert!(state.exists);
        assert!(state.session_wallet.is_some());
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

    impl Persister for MockPersister {
        fn exists(&self) -> bool {
            !self.wallet.borrow().is_empty()
        }
        fn get(&self) -> Result<String> {
            let s = self.wallet.borrow_mut();
            Ok(s[0].clone())
        }
        fn persist(&self, enc_wallet: &str) -> Result<()> {
            let mut w = self.wallet.borrow_mut();
            w.push(enc_wallet.to_owned());
            Ok(())
        }
    }
}
