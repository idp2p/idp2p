use super::jpm::Jpm;
use super::JwmHeader;
use idp2p_common::anyhow::Result;
use idp2p_common::base64url;
use idp2p_common::ed25519_dalek::{PublicKey, Signature, Verifier};
use idp2p_common::secret::EdSecret;
use idp2p_common::encode;
use idp2p_common::serde_json::json;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

const TYP: &str = "application/didcomm-signed+json";
const ALG: &str = "EdDSA";
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Jws {
    pub payload: String,
    pub signatures: Vec<JwsSignature>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct JwsSignature {
    pub protected: String,
    pub signature: String,
    pub header: JwmHeader,
}

impl JwsSignature {
    fn get_signature_bytes(&self) -> Result<[u8; 64]> {
        let sig_vec = base64url::decode_str(&self.signature)?;
        Ok(sig_vec.try_into().unwrap())
    }
}

impl Jws {
    pub fn new(jpm: Jpm, secret: EdSecret) -> Result<Jws> {
        let kid = format!("{}#{}", jpm.from, encode(&secret.to_publickey()));
        let header = JwmHeader { kid: kid };
        let payload_b64 = base64url::encode(jpm)?;
        let protected_json = json!({"typ": TYP.to_owned(), "alg": ALG.to_owned()});
        let protected_b64 = base64url::encode_str(&protected_json.to_string())?;
        let compact = format!("{protected_b64}.{payload_b64}");
        let sig_data = secret.sign_str(&compact);
        let sig_b64 = base64url::encode_bytes(&sig_data)?;
        let jws_signature = JwsSignature {
            protected: protected_b64,
            signature: sig_b64,
            header: header,
        };
        Ok(Jws {
            payload: payload_b64,
            signatures: vec![jws_signature],
        })
    }

    pub fn verify(&self, from_public: &[u8]) -> Result<bool> {
        let protected_json = json!({"typ": TYP.to_owned(), "alg": ALG.to_owned()});
        let protected_b64 = base64url::encode_str(&protected_json.to_string())?;
        let payload_b64 = self.payload.clone();
        let compact = format!("{protected_b64}.{payload_b64}");
        let public_key: PublicKey = PublicKey::from_bytes(from_public)?;
        let signature_bytes = self.signatures[0].get_signature_bytes()?;
        let signature = Signature::from(signature_bytes);
        public_key.verify(compact.as_bytes(), &signature)?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::didcomm::jwm::Jwm;
    use crate::json::didcomm::jwm::JwmBody;

    #[test]
    fn new_test() {
        let secret = EdSecret::new();
        let jwm = Jwm::new("from", "to", JwmBody::Message("body".to_owned()));
        let jws = Jws::new(Jpm::from(jwm), secret.clone()).unwrap();
        let r = jws.verify(&secret.to_publickey());
        assert!(r.is_ok());
    }
}
