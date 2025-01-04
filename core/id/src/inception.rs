use std::str::FromStr;

use idp2p_common::{cbor, id::Id};

use crate::{
    idp2p::id::{
        error::{IdError, IdInceptionError},
        types::IdInception,
    },
    IdProjection, PersistedIdInception, TIMESTAMP,
};

impl PersistedIdInception {
    pub(crate) fn verify(&self) -> Result<IdProjection, IdInceptionError> {
        let mut all_signers = vec![];
        let inception: IdInception = self.try_into()?;

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
            let signer_id = Id::from_str(signer.id.as_str()).map_err(|e| {
                IdInceptionError::InvalidSigner(IdError {
                    id: signer.id.clone(),
                    reason: e.to_string(),
                })
            })?;
            if signer_id.kind != "signer" {
                return Err(IdInceptionError::InvalidSigner(IdError {
                    id: signer.id.clone(),
                    reason: "invalid-signer-kind".to_string(),
                }))
            }
            signer_id.ensure(&signer.public_key).map_err(|e| {
                IdInceptionError::InvalidSigner(IdError {
                    id: signer.id.clone(),
                    reason: e.to_string(),
                })
            })?;
            if signers.contains(signer) {
                return Err(IdInceptionError::InvalidSigner(IdError {
                    id: signer.id.clone(),
                    reason: "duplicate-signer".to_string(),
                }));
            }
            all_signers.push(signer.id.clone());
            signers.push(signer.to_owned());
        }

        // Next Signer check
        //
        let total_next_signers = inception.next_signers.len() as u8;
        if total_next_signers < inception.next_threshold {
            return Err(IdInceptionError::ThresholdNotMatch);
        }
        let mut next_signers = vec![];
        for next_signer in &inception.next_signers {
            let next_signer_id = Id::from_str(next_signer.as_str()).map_err(|e| {
                IdInceptionError::InvalidNextSigner(IdError {
                    id: next_signer.clone(),
                    reason: e.to_string(),
                })
            })?;
            if next_signer_id.kind != "signer" {
                return Err(IdInceptionError::InvalidNextSigner(IdError {
                    id: next_signer.clone(),
                    reason: "invalid-next-signer-kind".to_string(),
                }));
            }
            if next_signers.contains(next_signer) {
                return Err(IdInceptionError::InvalidNextSigner(IdError {
                    id: next_signer.clone(),
                    reason: "duplicate-next-signer".to_string(),
                }));
            }
            all_signers.push(next_signer.clone());
            next_signers.push(next_signer.to_owned());
        }

        let mut claims = vec![];
        for claim in &inception.claims {
            claims.push(claim.to_owned());
        }

        let id_projection = IdProjection {
            id: self.id.clone(),
            event_id: self.id.clone(),
            event_timestamp: inception.timestamp,
            threshold: inception.threshold,
            signers: signers,
            next_threshold: inception.next_threshold,
            next_signers: next_signers,
            all_signers: all_signers,
            claims: claims,
            delegate_id: None
        };

        Ok(id_projection)
    }
}

impl TryFrom<&PersistedIdInception> for IdInception {
    type Error = IdInceptionError;

    fn try_from(value: &PersistedIdInception) -> Result<Self, Self::Error> {
        let id: Id = Id::from_str(value.id.as_str())
            .map_err(|e| IdInceptionError::InvalidId(e.to_string()))?;
        if id.kind != "id" {
            return Err(IdInceptionError::InvalidId(id.to_string()));
        }
        id.ensure(&value.payload)
            .map_err(|_| IdInceptionError::PayloadAndIdNotMatch)?;
        let inception: IdInception =
            cbor::decode(&value.payload).map_err(|_| IdInceptionError::InvalidPayload)?;
        Ok(inception)
    }
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::SigningKey;
    use idp2p_common::{CBOR_CODE, ED_CODE};
    use rand::rngs::OsRng;

    use crate::idp2p::id::types::IdSigner;

    use super::*;

    fn create_signer() -> IdSigner {
        let mut csprng = OsRng;
        let signing_key: SigningKey = SigningKey::generate(&mut csprng);
        let id = Id::new("signer", ED_CODE, signing_key.as_bytes())
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
        let id = Id::new("id", CBOR_CODE, inception_bytes.as_slice()).unwrap();
        eprintln!("ID: {}", id.to_string());
        let pinception = PersistedIdInception {
            id: id.to_string(),
            version: "".to_string(),
            payload: inception_bytes,
        };
        let result = pinception.verify();
        eprintln!("Result: {:#?}", result);
        assert!(result.is_ok());
    }
}
