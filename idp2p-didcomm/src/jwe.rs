use crate::jwk::Jwk;
use crate::jws::Jws;
use crate::JwmHeader;
use chacha20poly1305::aead::{Aead, NewAead};
use chacha20poly1305::ChaCha20Poly1305;
use chacha20poly1305::Key;
use chacha20poly1305::Nonce;
use idp2p_common::anyhow::Result;
use idp2p_common::decode;
use idp2p_common::decode_sized;
use idp2p_common::ed_secret::EdSecret;
use idp2p_common::log;
use idp2p_common::serde_json;
use idp2p_common::{base64url, encode};
use serde::{Deserialize, Serialize};

const TYP: &str = "application/didcomm-encrypted+json";
const ENC: &str = "XC20P";
const ALG: &str = "ECDH-ES+A256KW";

#[derive(Serialize, Deserialize, Clone)]
pub struct Jwe {
    pub iv: String,         // random initial vector 12 bytes
    pub ciphertext: String, // Encrypted message
    pub protected: String,
    pub recipients: Vec<JweRecipient>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct JweRecipient {
    pub header: JwmHeader,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct JweProtected {
    pub typ: String,
    pub enc: String,
    pub alg: String,
    pub epk: Jwk,
}

impl TryFrom<[u8; 32]> for JweProtected{
    type Error = idp2p_common::anyhow::Error;

    fn try_from(epk_bytes: [u8; 32]) -> Result<Self, Self::Error> {
        Ok(JweProtected {
            typ: TYP.to_owned(),
            enc: ENC.to_owned(),
            alg: ALG.to_owned(),
            epk: Jwk::try_from(epk_bytes)?,
        })
    }
}

impl Jwe {
    pub fn encrypt(jws: Jws, to: &str, to_public: [u8; 32]) -> Result<Jwe> {
        let enc_secret = EdSecret::new();
        let shared_secret = enc_secret.to_shared_secret(to_public);
        let iv = idp2p_common::create_random::<12>();
        let key = Key::from_slice(shared_secret.as_bytes());
        let cipher = ChaCha20Poly1305::new(&key);
        let nonce = Nonce::from_slice(&iv);
        let jws_str = serde_json::to_string(&jws)?;
        let ciphertext = cipher.encrypt(nonce, jws_str.as_ref()).unwrap();
        let protected: JweProtected = enc_secret.to_key_agreement().try_into()?;
        let to_kid = format!("{}#{}", to, encode(&to_public));
        let jwe = Jwe {
            protected: base64url::encode(&protected)?,
            iv: base64url::encode_bytes(&iv)?,
            ciphertext: base64url::encode_bytes(&ciphertext)?,
            recipients: vec![JweRecipient {
                header: JwmHeader { kid: to_kid },
            }],
        };
        Ok(jwe)
    }

    pub fn decrypt(&self, dec_secret: EdSecret) -> Result<String> {
        let protected_bytes = decode(&format!("u{}", self.protected));
        let protected_str = std::str::from_utf8(&protected_bytes)?;
        let protected: JweProtected = serde_json::from_str(protected_str)?;
        let epk_public = decode_sized::<32>(&format!("u{}", protected.epk.x))?;
        let shared_secret = dec_secret.to_shared_secret(epk_public);
        let key = Key::from_slice(shared_secret.as_bytes());
        let cipher = ChaCha20Poly1305::new(&key);
        let iv_b64 = decode(&format!("u{}", self.iv));
        let nonce = Nonce::from_slice(&iv_b64);
        let cipher_b64 = decode(&format!("u{}", self.ciphertext));
        let dec_result = cipher.decrypt(nonce, cipher_b64.as_ref());
        match dec_result {
            Ok(jws_bytes) => {
                let jws = std::str::from_utf8(&jws_bytes)?;
                return Ok(jws.to_owned());
            }
            Err(err) => {
                log::error!("{}", err);
                idp2p_common::anyhow::bail!("Decryption failed");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jpm::Jpm;
    use crate::jwm::{Jwm, JwmBody};
    #[test]
    fn jwe_protected_test() {
        let protected: JweProtected =[0; 32].try_into().unwrap();
        let jwk = Jwk::try_from([0; 32]).unwrap();
        assert_eq!(protected.alg, "ECDH-ES+A256KW");
        assert_eq!(protected.enc, "XC20P");
        assert_eq!(protected.typ, "application/didcomm-encrypted+json");
        assert_eq!(protected.epk, jwk);
    }
    #[test]
    fn jwe_encrypt_test() -> Result<()> {
        let from_secret = EdSecret::new();
        let to_secret = EdSecret::new();
        let jwm = Jwm::new("from", "to", JwmBody::Message("body".to_owned()));
        let jws = Jws::new(Jpm::from(jwm), from_secret.clone())?;
        let jwe = Jwe::encrypt(jws, "to", to_secret.to_key_agreement())?;
        let result = jwe.decrypt(to_secret.clone());
        assert!(result.is_ok());
        Ok(())
    }
}
