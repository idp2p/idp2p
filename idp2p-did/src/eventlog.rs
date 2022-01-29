use crate::{encode_vec, IdKey, IdKeyDigest};
use ed25519_dalek::{PublicKey, Signature, Signer, Verifier};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ProofStatement {
    #[serde(with = "encode_vec")]
    pub key: Vec<u8>,
    #[serde(with = "encode_vec")]
    pub value: Vec<u8>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct RecoverStatement {
    #[serde(rename = "keyType")]
    pub key_type: String,
    #[serde(with = "encode_vec")]
    #[serde(rename = "recoveryKeyDigest")]
    pub recovery_key_digest: IdKeyDigest,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct DocumentDigest {
    #[serde(with = "encode_vec")]
    pub value: Vec<u8>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(tag = "type")]
pub enum EventLogChange {
    #[serde(rename = "SetProof")]
    SetProof(ProofStatement),
    #[serde(rename = "SetRecoveryKey")]
    Recover(RecoverStatement),
    #[serde(rename = "SetDocument")]
    SetDocument(DocumentDigest),
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct EventLogPayload {
    pub previous: String,
    #[serde(with = "encode_vec")]
    #[serde(rename = "signerKey")]
    pub signer_key: IdKey,
    #[serde(with = "encode_vec")]
    #[serde(rename = "nextKeyDigest")]
    pub next_key_digest: IdKeyDigest,
    pub change: EventLogChange,
    pub timestamp: i64
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct EventLog {
    pub payload: EventLogPayload,
    #[serde(with = "encode_vec")]
    pub proof: Vec<u8>, // if recover assume recovery key
}

impl EventLogPayload {
    pub fn sign(&self, secret: &[u8]) -> Vec<u8> {
        let payload_json = serde_json::to_string(&self).unwrap();
        let bytes = payload_json.as_bytes();
        let keypair = crate::to_verification_keypair(secret);
        keypair.sign(&bytes).to_bytes().to_vec()
    }
}

impl EventLog {
    pub fn get_id(&self) -> String {
        crate::generate_cid(self)
    }

    pub fn verify(&self, public_data: &[u8]) -> bool {
        let payload_json = serde_json::to_string(&self.payload).unwrap();
        let bytes = payload_json.as_bytes();
        let public_key: PublicKey = PublicKey::from_bytes(&public_data).unwrap();
        let signature_bytes: [u8; 64] = self.proof.clone().try_into().unwrap();
        let signature = Signature::from(signature_bytes);
        public_key.verify(bytes, &signature).is_ok()
    }

    pub fn new(payload: EventLogPayload, proof: &[u8]) -> EventLog {
        let event_log = EventLog {
            payload: payload,
            proof: proof.to_vec(),
        };
        event_log
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use chrono::prelude::*;
    #[test]
    fn event_verify_test() {
        let secret = create_secret_key();
        let signer_key = to_verification_publickey(&secret);
        let payload = EventLogPayload {
            previous: "1".to_string(),
            signer_key: signer_key.clone(),
            next_key_digest: vec![],
            change: EventLogChange::SetDocument(DocumentDigest { value: vec![] }),
            timestamp: Utc::now().timestamp()
        };
        let proof = payload.sign(&secret);
        let log = EventLog::new(payload, proof);
        let is_valid = log.verify(&signer_key);
        assert!(is_valid);
    }

    #[test]
    fn event_create_test() {
        let secret = "bclc5pn2tfuhkqmupbr3lkyc5o4g4je6glfwkix6nrtf7hch7b3kq";
        let signer_key = to_verification_publickey(&multibase::decode(secret).unwrap().1);
        let timestamp = 0;
        let payload = EventLogPayload {
            previous: "1".to_string(),
            signer_key: signer_key.clone(),
            next_key_digest: hash(&signer_key),
            change: EventLogChange::SetDocument(DocumentDigest { value: vec![] }),
            timestamp: timestamp
        };
        let proof = payload.sign(&multibase::decode(secret).unwrap().1);
        let log = EventLog::new(payload, proof);
        let expected_json = r#"
        {
            "payload": {
                "previous": "1",
                "signerKey": "brgzkmbdnyevdth3sczvxjumd6bdl6ngn6eqbsbpazuvq42bfzk2a",
                "nextKeyDigest": "bcodiqdow4rvnu4o2wwtpv6dvjjsd63najdeazekh4w3s2dyb2tvq",
                "change": {"type": "SetDocument", "value": "b"},
                "timestamp": 0
            },
            "proof": "bvxrlrdqsehngru6c3k77d3a4cye7jis3yakkvqanb4btvg3la5a2cqchfpjmyotqhm3mye5j4dp27w2nwdp3tskwjvpnza3y6udg6cq"
        }"#;
        let expected: EventLog = serde_json::from_str(expected_json).unwrap();
        assert_eq!(serde_json::to_string(&log).unwrap(), serde_json::to_string(&expected).unwrap());
    }
}