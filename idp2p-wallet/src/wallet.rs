use crate::account::WalletAccount;
use crate::bip32::ExtendedSecretKey;
use chacha20poly1305::aead::{Aead, NewAead};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use derivation_path::ChildIndex;
use idp2p_common::anyhow;
use idp2p_common::anyhow::Result;
use idp2p_common::ed_secret::EdSecret;
use idp2p_common::serde_json;
use idp2p_core::did::Identity;
use idp2p_core::did_doc::CreateDocInput;
use idp2p_core::did_doc::IdDocument;
use idp2p_core::eventlog::DocumentDigest;
use idp2p_core::eventlog::EventLogChange;
use pbkdf2::{
    password_hash::{Error, PasswordHasher, SaltString},
    Pbkdf2,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Wallet {
    pub salt: [u8; 16],
    pub iv: [u8; 12],
    pub ciphertext: Vec<u8>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct CreateAccountResult {
    pub account: WalletAccount,
    pub did: Identity,
    pub next_index: u32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct WalletPayload {
    pub accounts: Vec<WalletAccount>,
    pub next_index: u32,
}

impl Wallet {
    pub fn new(password: &str) -> Result<Self> {
        let iv = idp2p_common::create_random::<12>();
        let payload = WalletPayload {
            accounts: vec![],
            next_index: 1000000000,
        };
        let salt_vec = idp2p_common::create_random::<16>();
        let enc_key_bytes = get_enc_key(password, &salt_vec).unwrap();
        let enc_key = Key::from_slice(&enc_key_bytes);
        let cipher = ChaCha20Poly1305::new(enc_key);
        let nonce = Nonce::from_slice(&iv);
        let ciphertext = cipher
            .encrypt(nonce, serde_json::to_string(&payload).unwrap().as_bytes())
            .expect("encryption failure!");
        let wallet = Wallet {
            salt: salt_vec,
            iv: iv,
            ciphertext: ciphertext,
        };
        Ok(wallet)
    }

    pub fn get_payload(&self, password: &str) -> Result<WalletPayload> {
        let enc_key_bytes = get_enc_key(password, &self.salt).unwrap();
        let enc_key = Key::from_slice(&enc_key_bytes);
        let cipher = ChaCha20Poly1305::new(enc_key);
        let nonce = Nonce::from_slice(&self.iv);
        let result = cipher.decrypt(nonce, self.ciphertext.as_ref()).unwrap();
        let payload: WalletPayload = serde_json::from_slice(&result).unwrap();
        Ok(payload)
    }
}

impl WalletPayload {
    pub fn create_account(&self, name: &str, seed: [u8; 16]) -> Result<CreateAccountResult> {
        if self.accounts.iter().any(|x| x.name == name) {
            anyhow::bail!("Account already exists")
        }

        let mut next_index = self.next_index;
        let inception_secret = derive_secret(seed, &mut next_index)?;
        let recovery_secret = derive_secret(seed, &mut next_index)?;
        let assertion_secret = derive_secret(seed, &mut next_index)?;
        let authentication_secret = derive_secret(seed, &mut next_index)?;
        let keyagreement_secret = derive_secret(seed, &mut next_index)?;
        let inception_key_digest = inception_secret.to_publickey_digest()?;
        let recovery_key_digest = recovery_secret.to_publickey_digest()?;
        let mut identity = Identity::new(&inception_key_digest, &recovery_key_digest);
        let create_doc_input = CreateDocInput {
            id: identity.id.clone(),
            assertion_key: assertion_secret.to_publickey().to_vec(),
            authentication_key: authentication_secret.to_publickey().to_vec(),
            keyagreement_key: keyagreement_secret.to_key_agreement().to_vec(),
        };
        let identity_doc = IdDocument::new(create_doc_input);
        let change = EventLogChange::SetDocument(DocumentDigest {
            value: identity_doc.get_digest(),
        });
        identity.document = Some(identity_doc);
        let next_secret = derive_secret(seed, &mut next_index)?;
        let next_digest = next_secret.to_publickey_digest()?;
        let signer = inception_secret.to_publickey();
        let payload = identity
            .microledger
            .create_event(&signer, &next_digest, change);
        let proof = inception_secret.sign(&payload);
        identity.microledger.save_event(payload, &proof);
        let account = WalletAccount {
            name: name.to_owned(),
            id: identity.id.clone(),
            assertion_secret: assertion_secret.to_bytes().to_vec(),
            authentication_secret: authentication_secret.to_bytes().to_vec(),
            keyagreement_secret: keyagreement_secret.to_bytes().to_vec(),
            next_secret: next_secret.to_bytes().to_vec(),
            credentials: None,
        };
        Ok(CreateAccountResult {
            account: account,
            did: identity,
            next_index: next_index,
        })
    }
}

fn get_enc_key(password: &str, salt: &[u8]) -> Result<Vec<u8>, Error> {
    let salt_b64 = idp2p_common::multibase::encode(idp2p_common::multibase::Base::Base64, salt);
    let salt = SaltString::new(&salt_b64[1..])?;
    let enc_key = Pbkdf2.hash_password(password.as_bytes(), &salt)?;
    let enc_key_hash = enc_key.hash.unwrap();
    Ok(enc_key_hash.as_bytes().to_vec())
}

fn derive_secret(seed: [u8; 16], derivation_index: &mut u32) -> Result<EdSecret> {
    let extended_secret = ExtendedSecretKey::from_seed(seed)?;
    let index = ChildIndex::hardened(derivation_index.clone()).unwrap();
    let key = extended_secret.derive_child(index)?;
    let secret = EdSecret::from(key.secret_key);
    *derivation_index += 1;
    Ok(secret)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_wallet_test() -> Result<()> {
        let password = "123456";
        let w = Wallet::new(password)?;
        let payload = w.get_payload(password)?;
        assert_eq!(payload.next_index, 1000000000);
        Ok(())
    }

    #[test]
    fn create_account_test() -> Result<()> {
        let password = "123456";
        let seed: [u8; 16] = idp2p_common::decode_sized("f000102030405060708090a0b0c0d0e0f")?;
        let w = Wallet::new(password)?;
        let payload = w.get_payload(password)?;
        let result = payload.create_account("ademcaglin", seed)?;
        assert_eq!(result.next_index, 1000000006);
        Ok(())
    }
}

/*
  pub fn set_document(&self, name: &str) -> Result<WalletCommandResult> {
        let account_r = self.accounts.iter().find(|x| x.name == name);
        if let Some(acc) = account_r {
            let next_index = self.derivation_index + 1;
            let assertion_index = next_index + 1;
            let authentication_index = assertion_index + 1;
            let keyagreement_index = authentication_index + 1;
            let next_secret_data = derive_key(self.seed, next_index)?;
            let assertion_secret_data = derive_key(self.seed, assertion_index)?;
            let authentication_secret_data = derive_key(self.seed, authentication_index)?;
            let keyagreement_secret_data = derive_key(self.seed, keyagreement_index)?;
            let next_secret = EdSecret::from(next_secret_data);
            let next_key_digest = next_secret.to_publickey_digest()?;
            let assertion_secret = EdSecret::from(assertion_secret_data);
            let authentication_secret = EdSecret::from(authentication_secret_data);
            let keyagreement_secret = EdSecret::from(keyagreement_secret_data);
            let create_doc_input = CreateDocInput {
                id: acc.id.clone(),
                assertion_key: assertion_secret.to_publickey().to_vec(),
                authentication_key: authentication_secret.to_publickey().to_vec(),
                keyagreement_key: keyagreement_secret.to_key_agreement().to_vec(),
            };
            let identity_doc = IdDocument::new(create_doc_input);
            return Ok(WalletCommandResult {
                account: WalletAccount {
                    next_derivation_index: next_index,
                    assertion_derivation_index: Some(assertion_index),
                    authentication_derivation_index: Some(authentication_index),
                    keyagreement_derivation_index: Some(keyagreement_index),
                    ..acc.clone()
                },
                derivation_index: keyagreement_index,
            });
        }
        anyhow::bail!("Account not found")
    }
*/
