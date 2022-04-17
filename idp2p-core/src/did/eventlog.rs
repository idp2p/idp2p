use idp2p_common::{
    ed25519_dalek::{PublicKey, Signature, Verifier},
    encode_vec, generate_cid, serde_json, IdKey, IdKeyDigest,
};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

use super::identity_doc::VerificationMethod;

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
    #[serde(rename = "masterKeyDigest")]
    pub recovery_key_digest: IdKeyDigest,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(tag = "type")]
pub enum EventLogChangeSet {
    SetProof(ProofStatement),
    SetAssertionKey(VerificationMethod),
    SetAuthenticationKey(VerificationMethod),
    SetAgreementKey(VerificationMethod),
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(tag = "type")]
pub enum EventLogChange {
    Set { sets: Vec<EventLogChangeSet> },
    Recover(RecoverStatement),
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
    pub timestamp: i64,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct EventLog {
    pub payload: EventLogPayload,
    #[serde(with = "encode_vec")]
    pub proof: Vec<u8>, // if recover assume recovery key
}

impl EventLog {
    pub fn get_id(&self) -> String {
        generate_cid(self)
    }

    pub fn verify(&self, public_data: &[u8]) -> bool {
        let payload_json = serde_json::to_string(&self.payload).unwrap();
        let bytes = payload_json.as_bytes();
        let public_key: PublicKey = PublicKey::from_bytes(&public_data).unwrap();
        let signature_bytes: [u8; 64] = self.proof.clone().try_into().unwrap();
        let signature = Signature::from(signature_bytes);
        public_key.verify(bytes, &signature).is_ok()
    }

    pub fn new(payload: EventLogPayload, proof: &[u8]) -> Self {
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
    use idp2p_common::chrono::prelude::*;
    use idp2p_common::{ed_secret::EdSecret, hash, serde_json};
    #[test]
    fn event_verify_test() {
        let secret = EdSecret::new();
        let signer_key = secret.to_publickey();
        let set_change = EventLogChangeSet::SetProof(ProofStatement {
            key: vec![],
            value: vec![],
        });
        let payload = EventLogPayload {
            previous: "1".to_string(),
            signer_key: signer_key.to_vec(),
            next_key_digest: vec![],
            change: EventLogChange::Set{ sets: vec![set_change]},
            timestamp: Utc::now().timestamp(),
        };
        let proof = secret.sign(&payload);
        let log = EventLog::new(payload, &proof);
        let is_valid = log.verify(&signer_key);
        assert!(is_valid);
    }

    #[test]
    fn event_create_test() {
        let secret_str = "bclc5pn2tfuhkqmupbr3lkyc5o4g4je6glfwkix6nrtf7hch7b3kq";
        let secret = EdSecret::from_str(secret_str).unwrap();
        let signer_key = secret.to_publickey();
        let timestamp = 0;
        let set_change = EventLogChangeSet::SetProof(ProofStatement {
            key: vec![],
            value: vec![],
        });
        let payload = EventLogPayload {
            previous: "1".to_string(),
            signer_key: signer_key.to_vec(),
            next_key_digest: hash(&signer_key),
            change: EventLogChange::Set{ sets: vec![set_change]},
            timestamp: timestamp,
        };
        let proof = secret.sign(&payload);
        let log = EventLog::new(payload, &proof);
        let expected_json = r#"
        {
            "payload": {
                "previous": "1",
                "signerKey": "brgzkmbdnyevdth3sczvxjumd6bdl6ngn6eqbsbpazuvq42bfzk2a",
                "nextKeyDigest": "bcodiqdow4rvnu4o2wwtpv6dvjjsd63najdeazekh4w3s2dyb2tvq",
                "change": { "type": "Set", "sets" :[{"type":"SetProof","key":"b","value":"b"}] },
                "timestamp": 0
            },
            "proof": "bqmnwfg52eb3qip4ebk76zz7yxt7q326odimfjcfna44nrdjluqqbhqe6wy6hlce5qjsbv7lrcyetpdc3usujqstkjbej2chbeaplmby"
        }"#;
        let expected: EventLog = serde_json::from_str(expected_json).unwrap();
        assert_eq!(
            serde_json::to_string(&log).unwrap(),
            serde_json::to_string(&expected).unwrap()
        );
    }
}
