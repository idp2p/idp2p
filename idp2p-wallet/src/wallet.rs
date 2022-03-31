use crate::derive_secret;
use crate::get_enc_key;
use crate::raw::RawWallet;
use crate::secret::SecretWallet;
use crate::session::WalletSession;
use crate::Persister;
use chacha20poly1305::aead::{Aead, NewAead};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use idp2p_common::{anyhow::Result, encode_vec, serde_json};
use idp2p_core::did::Identity;
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

impl<T> Wallet<T>
where
    T: Persister,
{
    pub fn register(
        &mut self,
        name: &str,
        password: &str,
        photo: &[u8],
    ) -> Result<(Identity, [u8; 16])> {
        let seed = idp2p_common::create_random::<16>();
        let iv = idp2p_common::create_random::<12>();
        let salt = idp2p_common::create_random::<16>();
        let mut next_index = 1000000000;
        let secret = derive_secret(seed, &mut next_index)?;
        let did = Identity::from_secret(secret.clone());
        let raw_wallet = RawWallet::new(name, did.clone(), photo);
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
        let wallet_str = self.persister.get()?;
        let enc_wallet = serde_json::from_str::<EncryptedWallet>(&wallet_str)?;
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
            let wallet_str = serde_json::to_string_pretty(&enc_wallet)?;
            self.persister.persist(&wallet_str);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;
    use idp2p_common::anyhow::Error;

    #[test]
    fn register_test() -> Result<()> {
        let mut wallet = Wallet {
            persister: MockPersister {
                wallet: RefCell::new(vec![]),
            },
            session: None,
        };
        wallet.register("Adem", "123456", &vec![])?;
        assert!(wallet.session.is_some());
        Ok(())
    }

    #[test]
    fn login_test() -> Result<()> {
        let mut wallet = Wallet {
            persister: MockPersister {
                wallet: RefCell::new(vec![]),
            },
            session: None,
        };
        wallet.register("Adem", "123456", &vec![])?;
        wallet.persist()?;
        wallet.session = None;
        wallet.login("123456")?;
        assert!(wallet.session.is_some());
        Ok(())
    }

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
        fn persist(&self, enc_wallet: &str) {
            let mut w = self.wallet.borrow_mut();
            w.push(enc_wallet.to_owned());
        }
    }
}
