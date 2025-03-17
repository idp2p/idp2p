use std::str::FromStr;

use idp2p_common::{cbor, identifier::Identifier};
use serde::{Deserialize, Serialize};

use crate::{
    error::IdError, IdSigner, IdState, IdClaim, IdResult, IdVersion, PersistedIdInception, TIMESTAMP
};

#[derive(Debug, Serialize, Deserialize)]
pub struct IdInception {
    pub timestamp: i64,
    pub threshold: u8,
    pub signers: Vec<IdSigner>,
    pub next_threshold: u8,
    pub next_signers: Vec<String>,
    pub claims: Vec<IdClaim>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PersistedIdInception {
    pub id: String,
    pub payload: Vec<u8>,
}

impl PersistedIdInception {
    pub(crate) fn verify(&self) -> Result<IdResult, IdError> {
        let mut all_signers = vec![];
        let inception: IdInception = self.try_into()?;

        // Timestamp check
        //
        if inception.timestamp < TIMESTAMP {
            return Err(IdError::new("invalid-timestamp"));
        }

        // Signer check
        //
        let total_signers = inception.signers.len() as u8;
        if total_signers < inception.threshold {
            return Err(IdError::new("threshold-not-match"));
        }
        let mut signers = vec![];
        for signer in &inception.signers {
            let signer_id = Identifier::from_str(signer.id.as_str())?;
            if signer_id.kind != "signer" {
                return Err(IdError::new("invalid-signer-kind"));
            }
            signer_id.ensure(&signer.public_key)?;
            if signers.contains(signer) {
                return Err(IdError::new("duplicate-signer"));
            }
            all_signers.push(signer.id.clone());
            signers.push(signer.to_owned());
        }

        // Next Signer check
        //
        let total_next_signers = inception.next_signers.len() as u8;
        if total_next_signers < inception.next_threshold {
            return Err(IdError::new("next-threshold-not-match"));
        }
        let mut next_signers = vec![];
        for next_signer in &inception.next_signers {
            let next_signer_id = Identifier::from_str(next_signer.as_str())?;
            if next_signer_id.kind != "signer" {
                return Err(IdError::new("invalid-next-signer-kind"));
            }
            if next_signers.contains(next_signer) {
                return Err(IdError::new("duplicate-next-signer"));
            }
            all_signers.push(next_signer.clone());
            next_signers.push(next_signer.to_owned());
        }

        let mut claims = vec![];
        for claim in &inception.claims {
            claims.push(claim.to_owned());
        }

        let id_state = IdState {
            id: self.id.clone(),
            event_id: self.id.clone(),
            event_timestamp: inception.timestamp,
            threshold: inception.threshold,
            signers: signers,
            next_threshold: inception.next_threshold,
            next_signers: next_signers,
            all_signers: all_signers,
            next_id: None,
        };
        let id_state_bytes = cbor::encode(&id_state);

        let id_result = IdResult {
            state: state,
            claims: vec![],
        };
        Ok(id_result)
    }
}

impl TryFrom<&PersistedIdInception> for IdInception {
    type Error = IdError;

    fn try_from(value: &PersistedIdInception) -> Result<Self, Self::Error> {
        let id = Identifier::from_str(value.id.as_str())?;
        if id.kind != "id" {
            return Err(IdError::new("invalid-id"));
        }
        id.ensure(&value.payload)?;
        let inception: IdInception = cbor::decode(&value.payload)?;
        Ok(inception)
    }
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::SigningKey;
    use idp2p_common::{CBOR_CODE, ED_CODE};
    use rand::rngs::OsRng;

    use super::*;

    fn create_signer() -> IdSigner {
        let mut csprng = OsRng;
        let signing_key: SigningKey = SigningKey::generate(&mut csprng);
        let id = Identifier::new("signer", ED_CODE, signing_key.as_bytes())
            .unwrap()
            .to_string();
        IdSigner {
            id: id,
            public_key: signing_key.to_bytes().to_vec(),
        }
    }

    #[test]
    fn test_verify_inception() {
        let inception = IdInception {
            timestamp: 1735689600,
            threshold: 1,
            signers: vec![create_signer()],
            next_threshold: 1,
            next_signers: vec![create_signer().id],
            claims: vec![],
        };
        let inception_bytes = cbor::encode(&inception);
        let id = Identifier::new("id", CBOR_CODE, inception_bytes.as_slice()).unwrap();
        eprintln!("ID: {}", id.to_string());
        let pinception = PersistedIdInception {
            id: id.to_string(),
            version: IdVersion { major: 1, minor: 0 },
            payload: inception_bytes,
        };
        let result = pinception.verify();
        eprintln!("Result: {:#?}", result);
        assert!(result.is_ok());
    }
}
