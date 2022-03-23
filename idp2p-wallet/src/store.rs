use crate::derive_secret;
use crate::get_enc_key;
use chacha20poly1305::aead::{Aead, NewAead};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use idp2p_common::anyhow::Result;
use idp2p_common::base64url;
use idp2p_common::ed_secret::EdSecret;
use idp2p_common::encode_vec;
use idp2p_common::serde_json;
use idp2p_core::did::Identity;
use idp2p_core::ver_cred::VerifiableCredential;
use idp2p_didcomm::jpm::Jpm;
use idp2p_didcomm::jwe::Jwe;
use idp2p_didcomm::jwm::Jwm;
use idp2p_didcomm::jws::Jws;
use idp2p_node::store::IdShared;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc::Sender;

pub enum WalletEvent {
    MessageReceived,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct EncryptedWallet {
    #[serde(with = "encode_vec")]
    pub salt: Vec<u8>,
    #[serde(with = "encode_vec")]
    pub iv: Vec<u8>,
    #[serde(with = "encode_vec")]
    pub ciphertext: Vec<u8>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Connection {
    /// Id of connection
    pub id: String,
    /// Username of connection
    pub username: String,
    /// Sent or Recieved messages
    pub messages: Vec<Message>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Message {
    pub sent: bool,
    pub text: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct WalletPayload {
    pub username: String,
    pub identity: Identity,
    next_index: u32,
    next_secret_index: u32,
    recovery_secret_index: u32,
    #[serde(with = "encode_vec")]
    assertion_secret: Vec<u8>,
    #[serde(with = "encode_vec")]
    authentication_secret: Vec<u8>,
    #[serde(with = "encode_vec")]
    keyagreement_secret: Vec<u8>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub connections: Vec<Connection>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub credentials: Vec<VerifiableCredential>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct WalletSession {
    pub payload: WalletPayload,
    created_at: i64,
    expire_at: i64,
    password: String,
    #[serde(with = "encode_vec")]
    salt: Vec<u8>,
    #[serde(with = "encode_vec")]
    iv: Vec<u8>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct WalletState {
    pub is_exist: bool,
    pub session: Option<WalletSession>,
}

pub struct WalletShared {
    pub state: Mutex<WalletState>,
    pub event_sender: Sender<WalletEvent>,
}

#[derive(Clone)]
pub struct WalletStore {
    pub wallet_path: PathBuf,
    pub shared: Arc<WalletShared>,
    pub id_shared: Arc<IdShared>,
}

pub struct WalletOptions {
    pub wallet_path: PathBuf,
    pub event_sender: Sender<WalletEvent>,
    pub id_shared: Arc<IdShared>,
}

impl WalletPayload {
    fn new(username: &str, seed: [u8; 16]) -> Result<Self> {
        let mut next_index = 1000000000;
        let secret = derive_secret(seed, &mut next_index)?;
        let did = Identity::from_secret(secret.clone());
        Ok(WalletPayload {
            next_index: next_index,
            username: username.to_owned(),
            identity: did,
            next_secret_index: next_index,
            recovery_secret_index: next_index,
            assertion_secret: secret.to_bytes().to_vec(),
            authentication_secret: secret.to_bytes().to_vec(),
            keyagreement_secret: secret.to_bytes().to_vec(),
            connections: vec![],
            credentials: vec![],
        })
    }
}

impl WalletSession {
    async fn persist(&self, wallet_path: &PathBuf) {
        let enc_key_bytes = get_enc_key(&self.password, &self.salt).unwrap();
        let enc_key = Key::from_slice(&enc_key_bytes);
        let cipher = ChaCha20Poly1305::new(enc_key);
        let nonce = Nonce::from_slice(&self.iv);
        let p_str = serde_json::to_string(&self.payload).unwrap();
        let ciphertext = cipher
            .encrypt(nonce, p_str.as_bytes())
            .expect("encryption failure!");
        let enc_wallet = EncryptedWallet {
            salt: self.salt.to_vec(),
            iv: self.iv.to_vec(),
            ciphertext: ciphertext,
        };
        let file = OpenOptions::new().write(true).open(wallet_path).unwrap();
        serde_json::to_writer_pretty(&file, &enc_wallet).unwrap();
    }
}

impl WalletStore {
    pub fn new(options: WalletOptions) -> Result<Self> {
        let state = WalletState {
            is_exist: options.wallet_path.is_file(),
            session: None,
        };
        let shared = Arc::new(WalletShared {
            state: Mutex::new(state),
            event_sender: options.event_sender,
        });
        if !std::path::Path::new(&options.wallet_path).exists() {
            std::fs::File::create(&options.wallet_path)?;
        }
        Ok(WalletStore {
            wallet_path: options.wallet_path,
            shared: shared,
            id_shared: options.id_shared,
        })
    }

    pub fn get_state(&self) -> Result<WalletState> {
        let state = self.shared.state.lock().unwrap();
        Ok(state.clone())
    }

    pub async fn create(&self, username: &str, password: &str) -> Result<Vec<u8>> {
        let seed = idp2p_common::create_random::<16>();
        let iv = idp2p_common::create_random::<12>();
        let salt_vec = idp2p_common::create_random::<16>();
        let payload = WalletPayload::new(username, seed)?;
        let enc_key_bytes = get_enc_key(password, &salt_vec).unwrap();
        let enc_key = Key::from_slice(&enc_key_bytes);
        let cipher = ChaCha20Poly1305::new(enc_key);
        let nonce = Nonce::from_slice(&iv);
        let p_str = serde_json::to_string(&payload)?;
        let ciphertext = cipher
            .encrypt(nonce, p_str.as_bytes())
            .expect("encryption failure!");
        let enc_wallet = EncryptedWallet {
            salt: salt_vec.to_vec(),
            iv: iv.to_vec(),
            ciphertext: ciphertext,
        };
        let file = OpenOptions::new()
            .write(true)
            .open(&self.wallet_path)
            .unwrap();
        serde_json::to_writer_pretty(&file, &enc_wallet).unwrap();
        self.login(password).await?;
        Ok(seed.to_vec())
    }

    pub async fn login(&self, password: &str) -> Result<()> {
        let mut file = File::open(self.wallet_path.as_path())?;
        let mut buff = String::new();
        file.read_to_string(&mut buff)?;
        let enc_wallet = idp2p_common::serde_json::from_str::<EncryptedWallet>(&buff)?;
        let enc_key_bytes = get_enc_key(password, &enc_wallet.salt).unwrap();
        let enc_key = Key::from_slice(&enc_key_bytes);
        let cipher = ChaCha20Poly1305::new(enc_key);
        let nonce = Nonce::from_slice(&enc_wallet.iv);
        let result = cipher
            .decrypt(nonce, enc_wallet.ciphertext.as_ref())
            .unwrap();
        let payload: WalletPayload = serde_json::from_slice(&result).unwrap();
        let session = WalletSession {
            payload: payload,
            created_at: 0,
            expire_at: 0,
            salt: enc_wallet.salt,
            iv: enc_wallet.iv,
            password: password.to_owned(),
        };
        let mut wallet = self.shared.state.lock().unwrap();
        wallet.session = Some(session);
        Ok(())
    }

    pub async fn logout(&self) {
        let mut wallet = self.shared.state.lock().unwrap();
        wallet.session = None;
    }

    pub async fn add_connection(&self, username: &str, id: &str) {
        let mut state = self.shared.state.lock().unwrap();
        if let Some(ref mut session) = state.session {
            let connection = Connection {
                id: id.to_owned(),
                username: username.to_owned(),
                messages: vec![],
            };
            session.payload.connections.push(connection);
            session.persist(&self.wallet_path).await;
        }
    }

    pub async fn send_message(&self, to: &str, message: &str) -> Result<()> {
        let mut state = self.shared.state.lock().unwrap();
        if let Some(ref mut session) = state.session {
            let id_state = self.id_shared.state.lock().unwrap();
            let to_did = id_state.entries.get(to).map(|entry| entry.clone()).unwrap();
            let jwm = Jwm::new(session.payload.identity.clone(), to_did.did, message);
            let enc_secret = EdSecret::from_bytes(&session.payload.keyagreement_secret.to_vec());
            let jwe = jwm.seal(enc_secret).unwrap();
            let json = idp2p_common::serde_json::to_string(&jwe).unwrap();
            // create a message
            // get
            // encrpt
            // send event
        }
        Ok(())
    }

    pub async fn handle_jwm(&self, jwm: &str) -> Result<()> {
        let mut state = self.shared.state.lock().unwrap();
        if let Some(ref mut session) = state.session {
            let doc = session.payload.identity.document.clone().unwrap();
            let jwe: Jwe = serde_json::from_str(jwm)?;
            if doc.get_first_keyagreement() != jwe.recipients[0].header.kid {
                idp2p_common::anyhow::bail!("INVALID_KID");
            }
            let dec_secret = EdSecret::from_bytes(&session.payload.keyagreement_secret);
            let json = jwe.decrypt(dec_secret)?;
            let jws: Jws = serde_json::from_str(&json)?;
            let jpm: Jpm = base64url::decode(&jws.payload)?;
            let id_state = self.id_shared.state.lock().unwrap();
            let from = id_state
                .entries
                .get(&jpm.from)
                .map(|entry| entry.clone())
                .unwrap();
            jws.verify(from.did).unwrap();
        }
        Ok(())
    }
}

async fn _listen_session_ttl() {
    // to do(remove session)
}
