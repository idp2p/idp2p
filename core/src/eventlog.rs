use crate::encode_me;
use crate::RecoveryKey;
use crate::SignerKey;
use ed25519_dalek::{PublicKey, Signature, Signer, Verifier};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ProofStatement {
    #[serde(with = "encode_me")]
    pub key: Vec<u8>,
    #[serde(with = "encode_me")]
    pub value: Vec<u8>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct RecoverStatement {
    pub next_signer_key: SignerKey,
    pub next_recovery_key: RecoveryKey,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct DocumentDigest {
    #[serde(with = "encode_me")]
    pub value: Vec<u8>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(tag = "type")]
pub enum EventLogChange {
    #[serde(rename = "set_proof")]
    SetProof(ProofStatement),
    #[serde(rename = "recover")]
    Recover(RecoverStatement),
    #[serde(rename = "set_document")]
    SetDocument(DocumentDigest),
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct EventLogPayload {
    pub previous: String, // if first = inception
    #[serde(with = "encode_me")]
    pub signer_publickey: Vec<u8>, // if recover = recover, else = signer_key
    pub change: EventLogChange,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct EventLog {
    pub payload: EventLogPayload,
    #[serde(with = "encode_me")]
    pub proof: Vec<u8>, // if recover assume recovery key
}

impl EventLog {
    pub fn get_id(&self) -> String {
        crate::generate_cid(self)
    }

    pub fn verify(&self, public_data: Vec<u8>) -> bool {
        let payload_json = serde_json::to_string(&self.payload).unwrap();
        let bytes = payload_json.as_bytes();
        let public_key: PublicKey = PublicKey::from_bytes(&public_data).unwrap();
        let signature_bytes: [u8; 64] = self.proof.clone().try_into().unwrap();
        let signature = Signature::from(signature_bytes);
        public_key.verify(bytes, &signature).is_ok()
    }

    pub fn new(payload: EventLogPayload, secret_key: Vec<u8>) -> EventLog {
        let payload_json = serde_json::to_string(&payload).unwrap();
        let keypair = crate::to_verification_keypair(secret_key.clone());
        let proof = keypair.sign(payload_json.as_bytes());
        let event_log = EventLog {
            payload: payload,
            proof: proof.to_bytes().to_vec(),
        };
        event_log
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    #[test]
    fn new_event() {
        let secret_key = create_secret_key();
        let signer_public = to_verification_publickey(secret_key.clone());
        let payload = EventLogPayload {
            previous: "1".to_string(),
            signer_publickey: signer_public.clone(),
            change: EventLogChange::SetDocument(DocumentDigest { value: vec![] }),
        };
        let log = EventLog::new(payload, secret_key);
        let is_valid = log.verify(signer_public);
        assert!(is_valid);
    }
}
