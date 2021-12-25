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
    #[serde(rename = "signerPublicKey")]
    pub signer_key: IdKey, 
    #[serde(with = "encode_vec")]
    #[serde(rename = "signerNextKey")]
    pub next_key_digest: IdKeyDigest,
    pub change: EventLogChange,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct EventLog {
    pub payload: EventLogPayload,
    #[serde(with = "encode_vec")]
    pub proof: Vec<u8>, // if recover assume recovery key
}

impl EventLogPayload{
    pub fn sign(&self, secret: &[u8]) -> Vec<u8>{
        let payload_json = serde_json::to_string(&self).unwrap();
        let keypair = crate::to_verification_keypair(secret);
        keypair.sign(payload_json.as_bytes()).to_bytes().to_vec()
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

    pub fn new(payload: EventLogPayload, proof: Vec<u8>) -> EventLog {
        let event_log = EventLog {
            payload: payload,
            proof: proof,
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
        let secret = create_secret_key();
        let signer_key = to_verification_publickey(&secret);
        let payload = EventLogPayload {
            previous: "1".to_string(),
            signer_key: signer_key.clone(),
            next_key_digest: vec![],
            change: EventLogChange::SetDocument(DocumentDigest { value: vec![] }),
        };
        let proof = payload.sign(&secret);
        let log = EventLog::new(payload, proof);
        let is_valid = log.verify(&signer_key);
        assert!(is_valid);
    }
}
