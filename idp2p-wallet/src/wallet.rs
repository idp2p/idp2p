use crate::account::WalletAccount;
use crate::bip32::ExtendedSecretKey;
use idp2p_common::anyhow::Result;
use chacha20poly1305::aead::{Aead, NewAead};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use derivation_path::ChildIndex;
use idp2p_common::encode;
use idp2p_common::ed_secret::EdSecret;
use idp2p_core::did::Identity;
use pbkdf2::{
    password_hash::{Error, PasswordHasher, SaltString},
    Pbkdf2,
};
use serde::{Deserialize, Serialize};
use idp2p_common::serde_json;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Wallet {
    pub salt: [u8; 16],
    pub iv: [u8; 12],
    pub ciphertext: Vec<u8>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct WalletPayload {
    pub seed: [u8; 16],
    pub accounts: Vec<WalletAccount>,
    pub derivation_index: u32,
}

impl Wallet {
    pub fn new(password: &str, seed: [u8;16]) -> Result<(Self, String)> {
        let iv = idp2p_common::create_random::<12>();
        let payload = WalletPayload {
            seed: seed,
            accounts: vec![],
            derivation_index: 1000000000,
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
        Ok((wallet, encode(&seed)))
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
    pub fn derive_key(&mut self) -> Result<([u8; 32], ChildIndex)> {
        let extended_secret = ExtendedSecretKey::from_seed(self.seed).unwrap();
        self.derivation_index += 1;
        let index = ChildIndex::hardened(self.derivation_index).unwrap();
        let key = extended_secret.derive_child(index).unwrap();
        Ok((key.secret_key, index))
    }

    pub fn create_account(&mut self, name: &str) -> Result<WalletAccount> {
        let (next_secret, next_index) = self.derive_key().unwrap();
        let (recovery_secret, recovery_index) = self.derive_key().unwrap();
        let next_key_digest = EdSecret::from(next_secret).to_publickey_digest()?;
        let recovery_key_digest = EdSecret::from(recovery_secret).to_publickey_digest()?;
        let identity = Identity::new(&next_key_digest, &recovery_key_digest);
        let account = WalletAccount {
            name: name.to_owned(),
            id: identity.id,
            credentials: vec![],
            next_derivation_index: next_index.to_u32(),
            recovery_derivation_index: recovery_index.to_u32(),
            assertion_derivation_index: None,
            authentication_derivation_index: None,
            keyagreement_derivation_index: None,
        };
        Ok(account)
    }
}

fn get_enc_key(password: &str, salt: &[u8]) -> Result<Vec<u8>, Error> {
    let salt = SaltString::new(&idp2p_common::encode_base64(&salt))?;
    let enc_key_hash = Pbkdf2
        .hash_password(password.as_bytes(), &salt)?
        .hash
        .unwrap();
    Ok(enc_key_hash.as_bytes().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn new_wallet_test() {
        let password = "123456";
        let seed = idp2p_common::decode("f000102030405060708090a0b0c0d0e0f");
        let (w, s) = Wallet::new(password, seed.try_into().unwrap()).unwrap();
        let payload = w.get_payload(password).unwrap();
        assert_eq!(encode(&payload.seed), s);
    }

    #[test]
    fn create_account_test() {
        let password = "123456";
        let seed = idp2p_common::decode("f000102030405060708090a0b0c0d0e0f");
        let (w, _) = Wallet::new(password, seed.try_into().unwrap()).unwrap();
        let mut payload = w.get_payload(password).unwrap();
        let acc = payload.create_account("ademcaglin").unwrap();
        println!("{:?}", acc);
    }
}

/*

let seed = idp2p_common::create_random::<16>();
        let mut mac = HmacSha512::new_varkey("idp2p seed".as_ref()).unwrap();
        mac.update(&seed);
        let bytes = mac.finalize().into_bytes().to_vec();
        let secret = bytes[..32].to_vec();
        let chain_code = bytes[32..64].to_vec();
        //let secret_key = SecretKey::from_bytes(&bytes[..32])?;
        //let chain_code = idp2p_common::create_random::<32>();
        let master_xpriv = "";
        let master_xpub = "";
        let next_secret = IdSecret::new();
        let signer_key = next_secret.to_verification_publickey();
        let next_key_digest = next_secret.to_publickey_digest();
        let recovery_key_digest = next_secret.to_publickey_digest();
        let mut identity = Identity::new(&next_key_digest, &recovery_key_digest);
        let create_doc_input = CreateDocInput {
            id: identity.id.clone(),
            assertion_key: next_secret.to_verification_publickey(),
            authentication_key: next_secret.to_verification_publickey(),
            keyagreement_key: next_secret.to_key_agreement_publickey(),
        };
        let identity_doc = IdDocument::new(create_doc_input);
        /*let change = identity.save_document(identity_doc);
        let payload = identity.microledger.create_event(&signer_key, &next_key_digest, change);
        let proof = next_secret.sign(&payload);
        identity.microledger.save_event(payload, &proof);
        let store = FileStore {};
        let account = Account {
            name: name.to_owned(),
            identity: identity.clone(),
            next_secret: next_secret.to_bytes(),
            authentication_secret: next_secret.to_bytes(),
            keyagreement_secret: next_secret.to_bytes(),
        };
        println!("Created identity: {:?}", identity.id.clone());
        store.put("identities", &identity.id.clone(), identity);
        store.put("accounts", name, account);*/
        Err(anyhow!(""))
use idp2p_common::encode;
use anyhow::Result;
use hmac::{Hmac, Mac, NewMac};
use sha2::Sha512;
use pbkdf2::{
    password_hash::{
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString,Error
    },
    Pbkdf2
};

type HmacSha512 = Hmac<Sha512>;

pub fn create_wallet() -> Vec<u8> {
    let seed = idp2p_common::create_random::<16>();
    let mut mac = HmacSha512::new_varkey("idp2p seed".as_ref()).unwrap();
    mac.update(&seed);
    mac.finalize().into_bytes().to_vec()
}

pub fn create_acc(password: &str) -> Result<bool, Error >{
    // plain seed
    // encrypted seed with pwd
    // plain keys
    // encrypted keys with pwd
    let seed = idp2p_common::create_random::<16>();
    let salt_str = encode(&idp2p_common::create_random::<16>());
    let salt = SaltString::new(&salt_str)?;
    let password_hash = Pbkdf2.hash_password(password.as_bytes(), &salt)?.to_string();
    let parsed_hash = PasswordHash::new(&password_hash)?;
    assert!(Pbkdf2.verify_password(password.as_bytes(), &parsed_hash).is_ok());
    Ok(true)
}
*/
