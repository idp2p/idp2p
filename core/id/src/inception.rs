use std::str::FromStr;

use idp2p_common::{cbor, identifier::Identifier};
use serde::{Deserialize, Serialize};

use crate::{
    error::IdInceptionError, types::{IdClaim, IdSigner, IdState}, TIMESTAMP
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IdInception {
    pub timestamp: i64,
    pub threshold: u8,
    pub signers: Vec<IdSigner>,
    pub next_threshold: u8,
    pub next_signers: Vec<String>,
    pub claims: Vec<IdClaim>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersistedIdInception {
    pub id: String,
    pub payload: Vec<u8>,
}


    pub(crate) fn verify(inception: &[u8]) -> Result<Vec<u8>, IdInceptionError> {
        let pinception: PersistedIdInception = cbor::decode(inception)?;

        let mut all_signers = vec![];
        let inception: IdInception = (&pinception).try_into()?;

        // Timestamp check
        //
        if inception.timestamp < TIMESTAMP {
            return Err(IdInceptionError::InvalidTimestamp);
        }

        // Signer check
        //
        let total_signers = inception.signers.len() as u8;
        if total_signers < inception.threshold {
            return Err(IdInceptionError::ThresholdNotMatch);
        }
        let mut signers = vec![];
        for signer in &inception.signers {
            let signer_id = Identifier::from_str(signer.id.as_str())?;
            if signer_id.kind != "signer" {
                return Err(IdInceptionError::InvalidSignerKind(signer.id.clone()));
            }
            signer_id.ensure(&signer.public_key)?;
            if signers.contains(signer) {
                return Err(IdInceptionError::DublicateSigner(signer.id.clone()));
            }
            all_signers.push(signer.id.clone());
            signers.push(signer.to_owned());
        }

        // Next Signer check
        //
        let total_next_signers = inception.next_signers.len() as u8;
        if total_next_signers < inception.next_threshold {
            return Err(IdInceptionError::NextThresholdNotMatch);
        }
        let mut next_signers = vec![];
        for next_signer in &inception.next_signers {
            let next_signer_id = Identifier::from_str(next_signer.as_str())?;
            if next_signer_id.kind != "signer" {
                return Err(IdInceptionError::InvalidNextSignerKind(next_signer.clone()));
            }
            if next_signers.contains(next_signer) {
                return Err(IdInceptionError::DublicateNextSigner(next_signer.clone()));
            }
            all_signers.push(next_signer.clone());
            next_signers.push(next_signer.to_owned());
        }

        let mut claims = vec![];
        for claim in &inception.claims {
            claims.push(claim.to_owned());
        }

        let id_state = IdState {
            id: pinception.id.clone(),
            event_id: pinception.id.clone(),
            event_timestamp: inception.timestamp,
            threshold: inception.threshold,
            signers: signers,
            next_threshold: inception.next_threshold,
            next_signers: next_signers,
            all_signers: all_signers,
            claims: claims,
            next_id: None,
            previous_id: None,
        };
        let id_state_bytes = cbor::encode(&id_state);

        Ok(id_state_bytes)
    }


impl TryFrom<&PersistedIdInception> for IdInception {
    type Error = IdInceptionError;

    fn try_from(value: &PersistedIdInception) -> Result<Self, Self::Error> {
        let id = Identifier::from_str(value.id.as_str())?;
        if id.kind != "id" {
            return Err(IdInceptionError::InvalidId(value.id.clone()));
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
            payload: inception_bytes,
        };
        let pinception_bytes = cbor::encode(&pinception);
        let result = verify(&pinception_bytes);
        eprintln!("Result: {:#?}", result);
        assert!(result.is_ok());
    }
}
