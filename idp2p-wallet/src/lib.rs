use idp2p_common::anyhow::Result;
use idp2p_common::ed_secret::EdSecret;
use crate::bip32::ExtendedSecretKey;
use derivation_path::ChildIndex;
use pbkdf2::{
    password_hash::{Error, PasswordHasher, SaltString},
    Pbkdf2,
};

pub(crate) fn get_enc_key(password: &str, salt: &[u8]) -> Result<Vec<u8>, Error> {
    let salt_b64 = idp2p_common::multibase::encode(idp2p_common::multibase::Base::Base64, salt);
    let salt = SaltString::new(&salt_b64[1..])?;
    let enc_key = Pbkdf2.hash_password(password.as_bytes(), &salt)?;
    let enc_key_hash = enc_key.hash.unwrap();
    Ok(enc_key_hash.as_bytes().to_vec())
}

pub(crate) fn derive_secret(seed: [u8; 16], derivation_index: &mut u32) -> Result<EdSecret> {
    let extended_secret = ExtendedSecretKey::from_seed(seed)?;
    let index = ChildIndex::hardened(derivation_index.clone()).unwrap();
    let key = extended_secret.derive_child(index)?;
    let secret = EdSecret::from(key.secret_key);
    *derivation_index += 1;
    Ok(secret)
}

pub mod bip32;
pub mod wallet;
pub mod store;
pub mod raw_wallet;
pub mod secret_wallet;
pub mod enc_wallet;
pub mod wallet_session;
pub mod store_;