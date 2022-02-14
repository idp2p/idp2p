use idp2p_common::encode;
use crate::jpm::Jpm;
use crate::JwmHeader;
use idp2p_common::anyhow::Result;
use idp2p_common::decode;
use idp2p_common::ed_secret::EdSecret;
use idp2p_common::serde_json;
use idp2p_common::serde_json::json;
use idp2p_core::did::Identity;
use serde::{Deserialize, Serialize};
use idp2p_common::base64url;

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

impl Jws {
    pub fn from(jpm: Jpm, secret: EdSecret) -> Result<Jws> {
        let kid = format!("{}#{}", jpm.from, encode(&secret.to_publickey()));
        let header = JwmHeader { kid: kid };
        let payload_str = jpm.body.as_str().unwrap();
        let payload_b64 = base64url::encode_str(payload_str)?;
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
        let payload_bytes = decode(&format!("u{}", self.payload));
        let jpm = serde_json::from_slice(&payload_bytes)?;
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jwm::Jwm;
    use idp2p_core::did::Identity;
    #[test]
    fn new_test() {
        let secret_str = "bd6yg2qeifnixj4x3z2fclp5wd3i6ysjlfkxewqqt2thie6lfnkma";
        let secret = EdSecret::from_str(secret_str).unwrap();
        let digest = secret.to_publickey_digest().unwrap();
        let from = Identity::new(&digest, &digest);
        let to = Identity::new(&vec![], &vec![]);
        let jwm = Jwm::new(from, to, r#"{ "body" : "body" }"#);
        let jws = Jws::from(Jpm::from(jwm), secret).unwrap();
        println!("{:?}", idp2p_common::serde_json::to_string(&jws));
    }
}
