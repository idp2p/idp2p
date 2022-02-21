use crate::account::WalletAccount;
use crate::account::WalletAccountDocument;
use crate::bip32::ExtendedSecretKey;
use chacha20poly1305::aead::{Aead, NewAead};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use derivation_path::ChildIndex;
use idp2p_common::anyhow;
use idp2p_common::anyhow::Result;
use idp2p_common::ed_secret::EdSecret;
use idp2p_common::encode_vec;
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

pub trait WalletStore {
    fn put_wallet(&self, key: &str, value: Wallet);
    fn get_wallet(&self, key: &str) -> Option<Wallet>;
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Wallet {
    #[serde(with = "encode_vec")]
    pub salt: Vec<u8>,
    #[serde(with = "encode_vec")]
    pub iv: Vec<u8>,
    #[serde(with = "encode_vec")]
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ciphertext: Vec<u8>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct AccountCommandResult {
    pub account: WalletAccount,
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
            salt: salt_vec.to_vec(),
            iv: iv.to_vec(),
            ciphertext: ciphertext,
        };
        Ok(wallet)
    }

    pub fn resolve(&self, password: &str) -> Result<WalletPayload> {
        let enc_key_bytes = get_enc_key(password, &self.salt).unwrap();
        let enc_key = Key::from_slice(&enc_key_bytes);
        let cipher = ChaCha20Poly1305::new(enc_key);
        let nonce = Nonce::from_slice(&self.iv);
        let result = cipher.decrypt(nonce, self.ciphertext.as_ref()).unwrap();
        let payload: WalletPayload = serde_json::from_slice(&result).unwrap();
        Ok(payload)
    }

    pub fn save(&mut self, password: &str, payload: WalletPayload) {
        let enc_key_bytes = get_enc_key(password, &self.salt).unwrap();
        let enc_key = Key::from_slice(&enc_key_bytes);
        let cipher = ChaCha20Poly1305::new(enc_key);
        let nonce = Nonce::from_slice(&self.iv);
        let ciphertext = cipher
            .encrypt(nonce, serde_json::to_string(&payload).unwrap().as_bytes())
            .expect("encryption failure!");
        self.ciphertext = ciphertext;
    }
}

impl WalletPayload {
    pub fn create_account(&self, name: &str, seed: [u8; 16]) -> Result<AccountCommandResult> {
        if self.accounts.iter().any(|x| x.name == name) {
            anyhow::bail!("Account already exists")
        }

        let mut next_index = self.next_index;
        let next_secret = derive_secret(seed, &mut next_index)?;
        let next_digest = next_secret.to_publickey_digest()?;
        let recovery_secret = derive_secret(seed, &mut next_index)?;
        let recovery_digest = recovery_secret.to_publickey_digest()?;
        let identity = Identity::new(&recovery_digest, &next_digest);
        let account = WalletAccount {
            name: name.to_owned(),
            did: identity.clone(),
            next_secret_index: next_index - 2,
            recovery_secret_index: next_index - 1,
            credentials: None,
            documents: None,
        };
        Ok(AccountCommandResult {
            account,
            next_index,
        })
    }

    pub fn set_document(&self, name: &str, seed: [u8; 16]) -> Result<AccountCommandResult> {
        let account_r = self.accounts.iter().find(|x| x.name == name);
        if let Some(acc) = account_r {
            let mut current_index = acc.next_secret_index;
            let current_secret = derive_secret(seed, &mut current_index)?;
            let signer = current_secret.to_publickey();

            let mut next_index = self.next_index;
            let assertion_secret = derive_secret(seed, &mut next_index)?;
            let authentication_secret = derive_secret(seed, &mut next_index)?;
            let keyagreement_secret = derive_secret(seed, &mut next_index)?;
            let create_doc_input = CreateDocInput {
                id: acc.did.id.clone(),
                assertion_key: assertion_secret.to_publickey().to_vec(),
                authentication_key: authentication_secret.to_publickey().to_vec(),
                keyagreement_key: keyagreement_secret.to_key_agreement().to_vec(),
            };
            let identity_doc = IdDocument::new(create_doc_input);
            let change = EventLogChange::SetDocument(DocumentDigest {
                value: identity_doc.get_digest(),
            });
            let mut new_acc = acc.clone();
            new_acc.next_secret_index = next_index;
            new_acc.did.document = Some(identity_doc.clone());
            let next_secret = derive_secret(seed, &mut next_index)?;
            let next_digest = next_secret.to_publickey_digest()?;
            let payload = new_acc
                .did
                .microledger
                .create_event(&signer, &next_digest, change);
            let proof = current_secret.sign(&payload);
            new_acc.did.microledger.save_event(payload, &proof);

            let acc_doc = WalletAccountDocument {
                assertion_secret: assertion_secret.to_bytes().to_vec(),
                authentication_secret: authentication_secret.to_bytes().to_vec(),
                keyagreement_secret: keyagreement_secret.to_bytes().to_vec(),
                document: identity_doc,
            };
            if new_acc.documents.is_none() {
                new_acc.documents = Some(vec![]);
            }
            let mut acc_docs = new_acc.documents.unwrap().clone();
            acc_docs.push(acc_doc);
            new_acc.documents = Some(acc_docs);
            return Ok(AccountCommandResult {
                account: new_acc,
                next_index,
            });
        }
        anyhow::bail!("Account not found")
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
        let payload = w.resolve(password)?;
        assert_eq!(payload.next_index, 1000000000);
        Ok(())
    }

    #[test]
    fn create_account_test() -> Result<()> {
        let password = "123456";
        let seed: [u8; 16] = idp2p_common::decode_sized("f000102030405060708090a0b0c0d0e0f")?;
        let w = Wallet::new(password)?;
        let payload = w.resolve(password)?;
        let result = payload.create_account("ademcaglin", seed)?;
        result.account.did.verify()?;
        assert_eq!(result.next_index, 1000000002);
        Ok(())
    }

    #[test]
    fn set_document_test() -> Result<()> {
        let password = "123456";
        let seed: [u8; 16] = idp2p_common::decode_sized("f000102030405060708090a0b0c0d0e0f")?;
        let w = Wallet::new(password)?;
        let mut payload = w.resolve(password)?;
        let r = payload.create_account("ademcaglin", seed)?;
        payload.accounts.push(r.account);
        payload.next_index = r.next_index;
        let result = payload.set_document("ademcaglin", seed)?;
        result.account.did.verify()?;
        assert_eq!(result.next_index, 1000000006);
        Ok(())
    }

    #[test]
    fn set_document2_test() -> Result<()> {
        let password = "123456";
        let seed: [u8; 16] = idp2p_common::decode_sized("f000102030405060708090a0b0c0d0e0f")?;
        let w = Wallet::new(password)?;
        let mut payload = w.resolve(password)?;
        let r = payload.create_account("ademcaglin", seed)?;
        payload.accounts.push(r.account);
        payload.next_index = r.next_index;
        let result = payload.set_document("ademcaglin", seed)?;
        result.account.did.verify()?;
        assert_eq!(result.next_index, 1000000006);
        Ok(())
    }
}
