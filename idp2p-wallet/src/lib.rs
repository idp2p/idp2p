use idp2p_common::encode;
use anyhow::*;
use hmac::{Hmac, Mac, NewMac};
use sha2::Sha512;
use pbkdf2::{
    password_hash::{
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
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

pub fn create_acc(password: &str) -> Result<bool, >{
    let seed = idp2p_common::create_random::<16>();
    let salt_str = format!("{}#{}", encode(&seed), password);
    let salt = SaltString::new(&salt_str).unwrap();
    let password_hash = Pbkdf2.hash_password(password.as_bytes(), &salt).unwrap().to_string();

    let parsed_hash = PasswordHash::new(&password_hash)?;
    assert!(Pbkdf2.verify_password(password, &parsed_hash).is_ok());
    Ok(true)
}

pub mod account;
pub mod bip32;