use alloc::collections::BTreeSet;
use chrono::{DateTime, Utc};
use ciborium::cbor;
use cid::Cid;
use core::str::FromStr;
use idp2p_common::{ED_CODE, cid::CidExt, error::CommonError, verification::ed25519};
use serde::{Deserialize, Serialize};

use crate::internal::{error::IdEventError, signer::IdSigner};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdProof {
    pub id: String,
    pub did: String,
    pub key_id: String,
    pub created: String,
    pub purpose: String,
    pub signature: Vec<u8>,
}

impl IdProof {
    pub fn verify(&self, payload: &[u8], signers: &BTreeSet<IdSigner>) -> Result<(), IdEventError> {
        // Validate created is RFC3339
        let _created: DateTime<Utc> = self
            .created
            .parse()
            .map_err(|_| IdEventError::invalid_proof(&self.key_id, "invalid created"))?;

        let kid = Cid::from_str(&self.key_id)?;
        let signer = signers
            .iter()
            .find(|s| s.id == self.key_id)
            .ok_or_else(|| IdEventError::InvalidSigner(self.key_id.clone()))?;

        // Ensure verification method CID matches the signer public key and codec
        if let Err(_e) = kid.ensure(&signer.public_key, vec![ED_CODE]) {
            return Err(IdEventError::invalid_proof(&self.key_id, "key mismatch"));
        }
        let data = cbor!({
            "did" => self.did.clone(),
            "key_id" => self.key_id.clone(),
            "created" => _created.timestamp(),
            "purpose" => self.purpose.clone(),
            "payload" => payload.clone(),
        })
        .map_err(|_| CommonError::EncodeError)?;
        let data_bytes = idp2p_common::cbor::encode(&data);

        match kid.codec() {
            ED_CODE => {
                if let Err(_e) = ed25519::verify(&signer.public_key, &data_bytes, &self.signature) {
                    return Err(IdEventError::invalid_proof(
                        &self.key_id,
                        "invalid signature",
                    ));
                }
            }
            _ => {
                return Err(IdEventError::InvalidSigner(
                    "Unsupported key type".to_string(),
                ));
            }
        }
        Ok(())
    }
}
