use crate::raw::RawWallet;
use crate::session::{SecretWallet, WalletSession};
use crate::WalletPersister;
use crate::{derive_secret, get_enc_key};
use chacha20poly1305::aead::{Aead, NewAead};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use idp2p_common::ed_secret::EdSecret;
use idp2p_common::log;
use idp2p_common::{anyhow::Result, encode_vec, serde_json};
use idp2p_core::did::Identity;
use idp2p_core::didcomm::jwe::Jwe;
use idp2p_core::didcomm::jwm::JwmBody;
use idp2p_core::didcomm::jws::Jws;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct PersistedWallet {
    pub raw: RawWallet,
    #[serde(with = "encode_vec")]
    pub ciphertext: Vec<u8>,
}

pub struct Wallet<T: WalletPersister> {
    pub persister: T,
    pub raw: Option<RawWallet>,
    pub session: Option<WalletSession>,
}

impl<T> Wallet<T>
where
    T: WalletPersister,
{
    pub fn persist(&self) -> Result<()> {
        let session = self.session.clone().unwrap();
        let raw = self.raw.clone().unwrap();
        let enc_key_bytes = get_enc_key(&session.password, &raw.salt).unwrap();
        let enc_key = Key::from_slice(&enc_key_bytes);
        let cipher = ChaCha20Poly1305::new(enc_key);
        let nonce = Nonce::from_slice(&raw.iv);
        let p_str = serde_json::to_string(&session.secret)?;
        let ciphertext = cipher
            .encrypt(nonce, p_str.as_bytes())
            .expect("encryption failure!");
        let persisted_wallet = PersistedWallet {
            raw: raw,
            ciphertext: ciphertext,
        };
        self.persister.persist_wallet(persisted_wallet)?;
        Ok(())
    }

    /*pub fn create_jwm(&mut self, to: Identity, jwm: JwmBody) -> Result<String> {
        let session = self.session.clone().unwrap();
        let jwm = self.raw.clone().unwrap().identity.new_jwm(to, jwm);
        let enc_secret = EdSecret::from_bytes(&session.secret.keyagreement_secret.to_vec());
        let jwe = jwm.seal(enc_secret)?;
        let json = idp2p_common::serde_json::to_string(&jwe)?;
        Ok(json)
    }*/

    /*pub fn resolve_jwe(&mut self, jwe_str: &str) -> Result<Jws> {
        let raw = self.raw.clone().unwrap();
        let session = self.session.clone().unwrap();
        let doc = raw.identity.document.clone().unwrap();
        let jwe: Jwe = serde_json::from_str(jwe_str)?;
        if doc.get_first_keyagreement() != jwe.recipients[0].header.kid {
            idp2p_common::anyhow::bail!("INVALID_KID");
        }
        let dec_secret = EdSecret::from_bytes(&session.secret.keyagreement_secret);
        let json = jwe.decrypt(dec_secret)?;
        let jws: Jws = serde_json::from_str(&json)?;
        Ok(jws)
    }*/
}

#[cfg(test)]
mod tests {
    use super::*;
    use idp2p_common::anyhow::Result;
    use std::cell::RefCell;

    /*struct MockPersister {
        wallet: RefCell<Vec<PersistedWallet>>,
    }

    impl WalletPersister for MockPersister {
        fn wallet_exists(&self) -> bool {
            todo!()
        }
        fn get_wallet(&self) -> Result<PersistedWallet> {
            let s = self.wallet.borrow_mut();
            Ok(s[0].clone())
        }
        fn persist_wallet(&self, enc_wallet: PersistedWallet) -> Result<()> {
            let mut w = self.wallet.borrow_mut();
            w.push(enc_wallet.to_owned());
            Ok(())
        }
    }*/
}
