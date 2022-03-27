use crate::derive_secret;
use crate::get_enc_key;
use crate::raw::RawWallet;
use crate::secret::SecretWallet;
use crate::session::WalletSession;
use chacha20poly1305::aead::{Aead, NewAead};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use idp2p_common::{anyhow::Result, encode_vec, serde_json};
use idp2p_core::did::Identity;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::path::PathBuf;

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
pub struct EncryptedWalletPayload {
    pub raw: RawWallet,
    pub secret: SecretWallet,
}

pub struct Wallet {
    pub path: PathBuf,
    pub session: Option<WalletSession>,
}

impl Wallet {
    pub fn register(&mut self, username: &str, password: &str) -> Result<(Identity, [u8;16])> {
        let seed = idp2p_common::create_random::<16>();
        let iv = idp2p_common::create_random::<12>();
        let salt = idp2p_common::create_random::<16>();
        let mut next_index = 1000000000;
        let secret = derive_secret(seed, &mut next_index)?;
        let did = Identity::from_secret(secret.clone());
        let raw_wallet = RawWallet::new(username, did.clone());
        let secret_wallet = SecretWallet {
            next_index: next_index,
            next_secret_index: next_index,
            recovery_secret_index: next_index,
            assertion_secret: secret.to_bytes().to_vec(),
            authentication_secret: secret.to_bytes().to_vec(),
            keyagreement_secret: secret.to_bytes().to_vec(),
        };

        self.session = Some(WalletSession {
            raw: raw_wallet,
            secret: secret_wallet,
            created_at: 0,
            expire_at: 0,
            password: password.to_owned(),
            salt: salt,
            iv: iv,
        });
        Ok((did, seed))
    }

    pub fn login(&mut self, password: &str) -> Result<()> {
        let mut file = File::open(self.path.as_path())?;
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
        });
        Ok(())
    }

    pub fn persist(&self) -> Result<()> {
        if let Some(ref session) = self.session {
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
        }
        Ok(())
    }
}
