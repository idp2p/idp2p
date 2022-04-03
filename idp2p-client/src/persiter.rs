use idp2p_common::anyhow::Result;
use idp2p_common::serde_json;
use idp2p_wallet::Persister;
use std::fs::{File, OpenOptions};
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;
use std::error::Error;
use serde::{Serialize, Deserialize};
use idp2p_common::encode_vec;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct IdConfig {
    #[serde(with = "encode_vec")]
    pub secret: Vec<u8>,
    pub listen_port: u16,
    pub node_addr: Option<String>,
}

impl IdConfig {
    pub fn new(secret: &[u8], port: u16) -> Self {
        Self {
            secret: secret.to_owned(),
            listen_port: port,
            node_addr: None,
        }
    }
}

pub struct FilePersister {
    path: PathBuf,
}

impl FromStr for FilePersister {
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            path: PathBuf::from_str(s)?,
        })
    }
    type Err = Box<dyn Error>;
}

impl Persister for FilePersister {
    fn exists(&self) -> bool {
        std::path::Path::new(&self.path).exists()
    }
    fn get(&self) -> Result<String> {
        let mut file = File::open(self.path.as_path())?;
        let mut buff = String::new();
        file.read_to_string(&mut buff)?;
        Ok(buff)
    }
    fn persist(&self, enc_wallet: &str) -> Result<()> {
        let file = OpenOptions::new().write(true).open(&self.path)?;
        serde_json::to_writer_pretty(&file, enc_wallet)?;
        Ok(())
    }
}



/*let mut file = File::open(self.path.as_path())?;
let mut buff = String::new();
file.read_to_string(&mut buff)?;
let enc_wallet = serde_json::from_str::<EncryptedWallet>(&buff)?;
let enc_key_bytes = get_enc_key(password, &enc_wallet.salt).unwrap();
let enc_key = Key::from_slice(&enc_key_bytes);
let cipher = ChaCha20Poly1305::new(enc_key);
let nonce = Nonce::from_slice(&enc_wallet.iv);
let result = cipher
    .decrypt(nonce, enc_wallet.ciphertext.as_ref())
    .unwrap();
let payload: EncryptedWalletPayload = serde_json::from_slice(&result)?;
self.session = Some(WalletSession {
    raw: payload.raw,
    secret: payload.secret,
    created_at: 0,
    expire_at: 0,
    password: password.to_owned(),
    salt: enc_wallet.salt.try_into().unwrap(),
    iv: enc_wallet.iv.try_into().unwrap(),
});*/

/*if let Some(ref session) = self.session {
    let enc_key_bytes = get_enc_key(&session.password, &session.salt).unwrap();
    let enc_key = Key::from_slice(&enc_key_bytes);
    let cipher = ChaCha20Poly1305::new(enc_key);
    let nonce = Nonce::from_slice(&session.iv);
    let p_str = serde_json::to_string(&EncryptedWalletPayload {
        raw: session.raw.clone(),
        secret: session.secret.clone(),
    })?;
    let ciphertext = cipher
        .encrypt(nonce, p_str.as_bytes())
        .expect("encryption failure!");
    let enc_wallet = EncryptedWallet {
        salt: session.salt.to_vec(),
        iv: session.iv.to_vec(),
        ciphertext: ciphertext,
    };
    let file = OpenOptions::new().write(true).open(&self.path)?;
    serde_json::to_writer_pretty(&file, &enc_wallet)?;
}*/
