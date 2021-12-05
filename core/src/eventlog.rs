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
#[serde(tag = "type")]
pub enum EventLogChange {
    #[serde(rename = "put_proof")]
    PutProof(ProofStatement),
    #[serde(rename = "recover")]
    Recover(RecoverStatement),
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct EventLogPayload {
    pub previous: String, // if first = inception
    #[serde(with = "encode_me")]
    pub signer_key: Vec<u8>, // if recover = recover, else = signer_key
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

    pub fn get_previous(&self) -> String {
        crate::generate_cid(self)
    }

    pub fn verify(&self, public_key: Vec<u8>) -> bool {
        let payload_json = serde_json::to_string(&self.payload).unwrap();
        let bytes = payload_json.as_bytes();
        let public_key: PublicKey = PublicKey::from_bytes(&public_key).unwrap();
        let signature_bytes: [u8; 64] = self.proof.clone().try_into().unwrap();
        let signature = Signature::from(signature_bytes);
        public_key.verify(bytes, &signature).is_ok()
    }

    pub fn create(payload: EventLogPayload, signer_key: Vec<u8>) -> EventLog {
        let payload_json = serde_json::to_string(&payload).unwrap();
        let keypair = crate::to_keypair(signer_key.clone());
        let proof = keypair.sign(payload_json.as_bytes());
        let event_log = EventLog {
            payload: payload,
            proof: proof.to_bytes().to_vec(),
        };
        event_log
    }
}
