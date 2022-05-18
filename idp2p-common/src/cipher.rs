use anyhow::Result;
use chacha20poly1305::aead::{Aead, NewAead};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use pbkdf2::{
    password_hash::{PasswordHasher, SaltString},
    Pbkdf2,
};

pub fn get_enc_key(password: &str, salt: &[u8]) -> anyhow::Result<Vec<u8>> {
    let salt_b64 = crate::multibase::encode(crate::multibase::Base::Base64, salt);
    let salt = SaltString::new(&salt_b64[1..]).map_err(|_| anyhow::anyhow!(""))?;
    let enc_key = Pbkdf2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| anyhow::anyhow!(""))?;
    let enc_key_hash = enc_key.hash.ok_or(anyhow::anyhow!(""))?;
    Ok(enc_key_hash.as_bytes().to_vec())
}

pub fn encrypt(enc_key_bytes: &[u8], iv: &[u8], content: &[u8]) -> Result<Vec<u8>> {
    let enc_key = Key::from_slice(&enc_key_bytes);
    let cipher = ChaCha20Poly1305::new(enc_key);
    let nonce = Nonce::from_slice(iv);
    let ciphertext = cipher
        .encrypt(nonce, content)
        .map_err(|_| anyhow::anyhow!(""))?;
    Ok(ciphertext)
}

pub fn decrypt(enc_key_bytes: &[u8], iv: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>> {
    let enc_key = Key::from_slice(&enc_key_bytes);
    let cipher = ChaCha20Poly1305::new(enc_key);
    let nonce = Nonce::from_slice(iv);
    let result = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| anyhow::anyhow!(""))?;
    Ok(result)
}
