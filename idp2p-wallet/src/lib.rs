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
    // let s = format!("Abc {password}");
    let seed = idp2p_common::create_random::<16>(); 
    let salt_str = encode(&idp2p_common::create_random::<16>());
    let salt = SaltString::new(&salt_str)?;
    let password_hash = Pbkdf2.hash_password(password.as_bytes(), &salt)?.to_string();
    let parsed_hash = PasswordHash::new(&password_hash)?;
    assert!(Pbkdf2.verify_password(password.as_bytes(), &parsed_hash).is_ok());
    Ok(true)
}

pub mod account;
pub mod bip32;