use std::collections::{BTreeMap, BTreeSet};

use alloc::str::FromStr;

use idp2p_common::{cbor, identifier::Identifier};
use serde::{Deserialize, Serialize};

use crate::{TIMESTAMP, error::IdInceptionError, state::IdState};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IdInception {
    pub timestamp: i64,
    pub threshold: u8,
    pub next_threshold: u8,
    pub signers: BTreeMap<String, Vec<u8>>,
    pub next_signers: BTreeSet<String>,
    pub claims: BTreeMap<String, Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersistedIdInception {
    pub id: String,
    pub payload: Vec<u8>,
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

pub(crate) fn verify(inception: &[u8]) -> Result<Vec<u8>, IdInceptionError> {
    let pinception: PersistedIdInception = cbor::decode(inception)?;

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

    let mut id_state = IdState {
        id: pinception.id.clone(),
        event_id: pinception.id.clone(),
        event_timestamp: inception.timestamp,
        threshold: inception.threshold,
        next_threshold: inception.next_threshold,
        signers: BTreeMap::new(),
        next_signers: BTreeSet::new(),
        all_signers: BTreeSet::new(),
        claims: BTreeMap::new(),
        next_id: None,
        previous_id: None,
    };

    for (signer_kid, signer_pk) in &inception.signers {
        let signer_id = Identifier::from_str(signer_kid)?;
        if signer_id.kind != "signer" {
            return Err(IdInceptionError::InvalidSignerKind(signer_id.to_string()));
        }
        signer_id.ensure(&signer_pk)?;
        if id_state.signers.contains_key(signer_kid) {
            return Err(IdInceptionError::DublicateSigner(signer_id.to_string()));
        }
        id_state.all_signers.insert(signer_id.to_string());
        id_state
            .signers
            .insert(signer_kid.to_string(), signer_pk.to_owned());
    }

    // Next Signer check
    //
    let total_next_signers = inception.next_signers.len() as u8;
    if total_next_signers < inception.next_threshold {
        return Err(IdInceptionError::NextThresholdNotMatch);
    }
    for next_signer in &inception.next_signers {
        let next_signer_id = Identifier::from_str(next_signer)?;
        if next_signer_id.kind != "signer" {
            return Err(IdInceptionError::InvalidNextSignerKind(next_signer.clone()));
        }
        if id_state.next_signers.contains(next_signer) {
            return Err(IdInceptionError::DublicateNextSigner(next_signer.clone()));
        }
        id_state.all_signers.insert(next_signer.to_owned());
        id_state.next_signers.insert(next_signer.to_owned());
    }

    for (claim_key, claim_event ) in &inception.claims {
        id_state.claims.insert(claim_key.to_owned(), vec![claim_event.to_owned()]);
    }

    let id_state_bytes = cbor::encode(&id_state);

    Ok(id_state_bytes)
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::SigningKey;
    use idp2p_common::{CBOR_CODE, ED_CODE};
    use rand::rngs::OsRng;

    use super::*;

    fn create_signer() -> (String, Vec<u8>) {
        let mut csprng = OsRng;
        let signing_key: SigningKey = SigningKey::generate(&mut csprng);
        let id = Identifier::new("signer", ED_CODE, signing_key.as_bytes())
            .unwrap()
            .to_string();
        (id, signing_key.to_bytes().to_vec())
    }

    #[test]
    fn test_verify_inception() {
        let (kid, pk) = create_signer();
        let mut signers = BTreeMap::new();
        let mut next_signers = BTreeSet::new();

        signers.insert(kid.clone(), pk);
        next_signers.insert(kid);
        let inception = IdInception {
            timestamp: 1735689600,
            threshold: 1,
            signers: signers,
            next_threshold: 1,
            next_signers: next_signers,
            claims: BTreeMap::new(),
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
        assert!(result.is_ok());
        let result: IdState = cbor::decode(&result.unwrap()).unwrap();
        eprintln!("Result: {:#?}", result);
    }
}
