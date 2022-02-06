use crate::jwk::Jwk;
use crate::jwm::Jwm;
use crate::jws::Jws;
use crate::JwmHeader;
use anyhow::Result;
use chacha20poly1305::aead::{Aead, NewAead};
use chacha20poly1305::ChaCha20Poly1305;
use chacha20poly1305::Key;
use chacha20poly1305::Nonce;
use idp2p_common::secret::IdSecret;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Jwe {
    pub iv: String,         // random initial vector 12 bytes
    pub ciphertext: String, // Encrypted message
    pub protected: String,
    pub recipients: Vec<JweRecipient>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct JweRecipient {
    header: JwmHeader,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct JweProtected {
    pub typ: String,
    pub enc: String,
    pub alg: String,
    pub epk: Jwk,
}

impl JweProtected {
    fn new(epk_bytes: &[u8]) -> JweProtected {
        JweProtected {
            typ: "application/didcomm-encrypted+json".to_owned(),
            enc: "XC20P".to_owned(),
            alg: "ECDH-ES+A256KW".to_owned(),
            epk: Jwk::from_bytes(epk_bytes),
        }
    }
}

impl Jwe {
    pub fn from(jwm: Jwm, secret: IdSecret) -> Result<Jwe> {
        let jws = Jws::from(jwm.clone(), secret);
        let raw = serde_json::to_string(&jws).unwrap();
        let enc_secret = IdSecret::new();
        let to_doc = jwm.to.document.unwrap();
        let to_kid = to_doc.get_first_keyagreement();
        let to_public = to_doc.get_verification_method(&to_kid).unwrap();
        let shared_secret = enc_secret.to_shared_secret(&to_public.bytes);
        let iv = idp2p_common::create_random::<12>();
        let key = Key::from_slice(shared_secret.as_bytes());
        let cipher = ChaCha20Poly1305::new(&key);
        let nonce = Nonce::from_slice(&iv);
        let ciphertext = cipher.encrypt(nonce, raw.as_ref()).unwrap();
        let protected = JweProtected::new(&enc_secret.to_key_agreement_publickey());
        let protected_str = serde_json::to_string(&protected).unwrap();
        let jwe = Jwe {
            protected: idp2p_common::encode_base64url(protected_str.as_bytes()),
            iv: idp2p_common::encode_base64url(&iv),
            ciphertext: idp2p_common::encode_base64url(&ciphertext),
            recipients: vec![],
        };
        Ok(jwe)
    }
}
