use crate::jwm::Jwm;
use idp2p_common::secret::IdSecret;
use crate::JwmHeader;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Jws {
    pub payload: String,
    pub signatures: Vec<JwsSignature>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct JwsSignature{
   pub protected: String,
   pub signature: String,
   pub header: JwmHeader
}

impl Jws{
    
    pub fn from(jwm: Jwm, secret: IdSecret) -> Jws{
        let protected = r#"{"typ":"application/didcomm-signed+json","alg":"EdDSA"}"#;
        //let payload = idp2p_common::encode_base64url(value: &[u8])(serde_json::to_string(&self.to_jpm()));
        Jws{
            payload: "payload".to_owned(),
            signatures: vec![]
        }
    }
}
