use crate::jpm::Jpm;
use crate::JwmHeader;
use idp2p_common::anyhow::Result;
use idp2p_common::base64url;
use idp2p_common::decode;
use idp2p_common::ed25519_dalek::{PublicKey, Signature, Verifier};
use idp2p_common::ed_secret::EdSecret;
use idp2p_common::encode;
use idp2p_common::serde_json;
use idp2p_common::serde_json::json;
use idp2p_core::did::Identity;
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

    pub fn verify(&self, from: Identity) -> Result<bool> {
        let protected_json = json!({"typ": TYP.to_owned(), "alg": ALG.to_owned()});
        let protected_b64 = base64url::encode_str(&protected_json.to_string())?;
        let payload_b64 = self.payload.clone();
        let compact = format!("{protected_b64}.{payload_b64}");
        let doc = from.document.expect("Document not found");
        let ver_method = doc
            .get_verification_method(&self.signatures[0].header.kid)
            .expect("Public key not found");
        let public_key: PublicKey = PublicKey::from_bytes(&ver_method.bytes)?;
        let signature_bytes = self.signatures[0].get_signature_bytes()?;
        let signature = Signature::from(signature_bytes);
        public_key.verify(compact.as_bytes(), &signature)?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jwm::Jwm;
    use idp2p_core::did::Identity;
    use idp2p_core::did_doc::CreateDocInput;
    use idp2p_core::did_doc::IdDocument;
    use idp2p_core::eventlog::DocumentDigest;
    use idp2p_core::eventlog::EventLogChange;
    #[test]
    fn new_test() {
        let secret_str = "bd6yg2qeifnixj4x3z2fclp5wd3i6ysjlfkxewqqt2thie6lfnkma";
        let secret = EdSecret::from_str(secret_str).unwrap();
        let digest = secret.to_publickey_digest().unwrap();
        let mut from = Identity::new(&digest, &digest);
        let mut to = Identity::new(&digest, &digest);
        save_doc(&mut from, secret.clone());
        save_doc(&mut to, secret.clone());
        let jwm = Jwm::new(from.clone(), to, r#"{ "body" : "body" }"#);
        let jws = Jws::new(Jpm::from(jwm), secret).unwrap();
        let r = jws.verify(from);
        assert!(r.is_ok());
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
        let next = secret.to_publickey_digest().unwrap();
        let payload = did.microledger.create_event(&signer, &next, change);
        let proof = secret.sign(&payload);
        did.microledger.save_event(payload, &proof);
    }
}
