use crate::jwk::Jwk;
use crate::jws::Jws;
use crate::JwmHeader;
use chacha20poly1305::aead::{Aead, NewAead};
use chacha20poly1305::ChaCha20Poly1305;
use chacha20poly1305::Key;
use chacha20poly1305::Nonce;
use idp2p_common::anyhow::Result;
use idp2p_common::base64url;
use idp2p_common::decode;
use idp2p_common::decode_sized;
use idp2p_common::ed_secret::EdSecret;
use idp2p_common::serde_json;
use idp2p_core::did::Identity;
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

impl JweProtected {
    fn new(epk_bytes: [u8; 32]) -> Result<JweProtected> {
        Ok(JweProtected {
            typ: TYP.to_owned(),
            enc: ENC.to_owned(),
            alg: ALG.to_owned(),
            epk: Jwk::from_bytes(epk_bytes)?,
        })
    }
}

impl Jwe {
    pub fn encrypt(jws: Jws, to: Identity) -> Result<Jwe> {
        let enc_secret = EdSecret::new();
        let to_doc = to.document.expect("The id document not found");
        let to_kid = to_doc.get_first_keyagreement();
        let to_ver = to_doc
            .get_verification_method(&to_kid)
            .expect("Key not found");
        let to_public: [u8; 32] = to_ver.bytes.try_into().expect("Key should be 32 bytes");
        let shared_secret = enc_secret.to_shared_secret(to_public);
        let iv = idp2p_common::create_random::<12>();
        let key = Key::from_slice(shared_secret.as_bytes());
        let cipher = ChaCha20Poly1305::new(&key);
        let nonce = Nonce::from_slice(&iv);
        let jws_str = serde_json::to_string(&jws)?;
        let ciphertext = cipher.encrypt(nonce, jws_str.as_ref()).unwrap();
        let protected = JweProtected::new(enc_secret.to_key_agreement())?;
        let jwe = Jwe {
            protected: base64url::encode(&protected)?,
            iv: base64url::encode_bytes(&iv)?,
            ciphertext: base64url::encode_bytes(&ciphertext)?,
            recipients: vec![],
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
        let jws_bytes = cipher.decrypt(nonce, cipher_b64.as_ref()).unwrap();
        let jws = std::str::from_utf8(&jws_bytes)?;
        Ok(jws.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jpm::Jpm;
    use crate::jwm::JwmBody;
    use crate::jwm::JwmHandler;
    use idp2p_core::did_doc::CreateDocInput;
    use idp2p_core::did_doc::IdDocument;
    use idp2p_core::eventlog::DocumentDigest;
    use idp2p_core::eventlog::EventLogChange;
    #[test]
    fn jwe_protected_test() {
        let protected = JweProtected::new([0; 32]).unwrap();
        let jwk = Jwk::from_bytes([0; 32]).unwrap();
        assert_eq!(protected.alg, "ECDH-ES+A256KW");
        assert_eq!(protected.enc, "XC20P");
        assert_eq!(protected.typ, "application/didcomm-encrypted+json");
        assert_eq!(protected.epk, jwk);
    }
    #[test]
    fn jwe_encrypt_test() -> Result<()> {
        let (mut from_id, from_secret) = create_did();
        save_doc(&mut from_id, from_secret.clone());

        let (mut to_id, to_secret) = create_did();
        save_doc(&mut to_id, to_secret.clone());
        let jwm = from_id.new_jwm(to_id.clone(), JwmBody::Message("body".to_owned()));
        let jws = Jws::new(Jpm::from(jwm), from_secret.clone())?;
        let jwe = Jwe::encrypt(jws, to_id.clone())?;
        let result = jwe.decrypt(to_secret.clone())?;
        println!("result: {:?}", result);
        Ok(())
    }

    fn create_did() -> (Identity, EdSecret) {
        let secret = EdSecret::new();
        let ed_key_digest = secret.to_publickey_digest().unwrap();
        (Identity::new(&ed_key_digest, &ed_key_digest), secret)
    }

    fn save_doc(did: &mut Identity, secret: EdSecret) {
        let ed_key = secret.to_publickey();
        let x_key = secret.to_key_agreement();
        let input = CreateDocInput {
            id: did.id.clone(),
            assertion_key: ed_key.to_vec(),
            authentication_key: ed_key.to_vec(),
            keyagreement_key: x_key.to_vec(),
        };
        let doc = IdDocument::new(input);
        let doc_digest = doc.get_digest();
        did.document = Some(doc);
        let change = EventLogChange::SetDocument(DocumentDigest { value: doc_digest });
        let signer = secret.to_publickey();
        let next_digest = secret.to_publickey_digest().unwrap();
        let payload = did.microledger.create_event(&signer, &next_digest, change);
        let proof = secret.sign(&payload);
        did.microledger.save_event(payload, &proof);
    }
}
