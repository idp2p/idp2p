use crate::wallet::{Wallet, WalletPersister};
use crate::wallet_raw::{ RawWallet, SharedWallet};
use crate::wallet_session::{ SecretWallet, SessionState, WalletSession};
use idp2p_common::anyhow::Result;
use idp2p_common::bip32::derive_secret;
use idp2p_common::{decrypt, encode_vec, get_enc_key, log, serde_json};
use idp2p_core::did::identity::Identity;
use idp2p_core::didcomm::jpm::Jpm;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum WalletCommand {
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
    Logout,
    Connect {
        id: String,
    },
    Accept{
        id: String,
    },
    SendMessage {
        id: String,
        msg: String,
    },
}
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct WalletState {
    #[serde(flatten)]
    pub raw: RawWallet,
    pub session: Option<SessionState>,
}

pub struct WalletStore {
    pub wallet: Mutex<Wallet>,
}

impl WalletStore {
    pub fn new<T: WalletPersister>(persister: T) -> Self {
        let mut wallet = Wallet {
            raw: None,
            session: None,
        };
        if persister.wallet_exists() {
            log::info!("Persisted wallet is importing");
            wallet.raw = Some(persister.get_wallet().unwrap().raw);
        }
        WalletStore {
            wallet: Mutex::new(wallet),
        }
    }

    pub fn handle_command<T: WalletPersister>(&self, cmd: WalletCommand, persister: T) -> Result<WalletState>{
        match cmd{
            WalletCommand::Get => {
                let wallet = self.wallet.lock().unwrap();
                if let Some(ref raw) = wallet.raw {
                    if let Some(ref session) = wallet.session {
                        /*return Some(WalletState {
                            raw: raw.clone(),
                            session: Some(session.to_state()),
                        });*/
                    }
                    /*return Some(WalletState {
                        raw: raw.clone(),
                        session: None,
                    });*/
                }
            }
            WalletCommand::Register { name, photo, password } =>{
                let mut wallet = self.wallet.lock().unwrap();
                if wallet.raw.is_none() {
                    let seed = idp2p_common::create_random::<16>();
                    let mut next_index = 1000000000;
                    let secret = derive_secret(seed, &mut next_index)?;
                    let did = Identity::from_secret(secret.clone());
                    let shared = SharedWallet {
                        next_index: next_index,
                        next_secret_index: 1000000000,
                        recovery_secret_index: 1000000000,
                        assertion_secret_index: 1000000000,
                        authentication_secret_index: 1000000000,
                        agreement_secret_index: 1000000000,
                    };
                    wallet.raw = Some(RawWallet::new(&name, &photo, did.id.as_str(), shared)?);
                    wallet.session = Some(WalletSession::new_with_secret(secret, &password));
                }
                idp2p_common::anyhow::bail!("Identity already exists")
            }
            WalletCommand::Connect { id } => {

            }
            WalletCommand::Accept { id } => {
                
            }
            WalletCommand::SendMessage { id, msg } =>{
                
            }
            WalletCommand::Login { password } => {
                let mut wallet = self.wallet.lock().unwrap();
                let raw = wallet.raw.clone().unwrap();
                let persisted_wallet = persister.get_wallet()?;
                let enc_key_bytes = get_enc_key(&password, &raw.salt).unwrap();
                let result = decrypt(
                    &enc_key_bytes,
                    &raw.iv,
                    persisted_wallet.ciphertext.as_ref(),
                )?;
                let secret_wallet: SecretWallet = serde_json::from_slice(&result)?;
                wallet.session = Some(WalletSession::new(secret_wallet, &password));
            }
            WalletCommand::Logout => {
                let mut wallet = self.wallet.lock().unwrap();
                wallet.session = None;
            }
            _ => {}
        }
        idp2p_common::anyhow::bail!("")
    }

    pub fn get_agreement_secret(&self) -> Vec<u8> {
        let wallet = self.wallet.lock().unwrap();
        if let Some(session) = wallet.session.clone() {
            return session.secret.keyagreement_secret;
        }
        vec![]
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

/*use crate::IdCommand;
use idp2p_common::anyhow::Result;

pub async fn handle(cmd: IdCommand) -> Result<Option<WalletState>> {
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


}

*/
