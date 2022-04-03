use crate::get_enc_key;
use crate::raw::RawWallet;
use crate::secret::SecretWallet;
use crate::session::WalletSession;
use crate::Persister;
use chacha20poly1305::aead::{Aead, NewAead};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use idp2p_common::{anyhow::Result, encode_vec, serde_json};
use serde::{Deserialize, Serialize};

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

pub struct Wallet<T: Persister> {
    pub persister: T,
    pub session: Option<WalletSession>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct WalletState {
    pub exists: bool,
    pub session: Option<SessionState>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct SessionState {
    pub created_at: i64,
    pub expire_at: i64,
    pub raw_wallet: RawWallet,
}

impl<T> Wallet<T>
where
    T: Persister,
{
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
            let wallet_str = serde_json::to_string_pretty(&enc_wallet)?;
            self.persister.persist(&wallet_str)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use idp2p_common::anyhow::Result;
    use std::cell::RefCell;

    struct MockPersister {
        wallet: RefCell<Vec<String>>,
    }

    impl Persister for MockPersister {
        fn exists(&self) -> bool {
            todo!()
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
