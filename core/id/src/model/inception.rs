use alloc::collections::{BTreeMap, BTreeSet};

use super::{signer::IdSigner, state::IdState};
use crate::{VALID_FROM, error::IdEventError, model::envelope::IdEventEnvelope};
use alloc::str::FromStr;
use chrono::{DateTime, TimeZone, Utc};
use cid::Cid;
use idp2p_common::{ED_CODE, cbor, cid::CidExt};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IdInception {
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub prior_id: Option<String>,
    pub threshold: u8,
    pub next_threshold: u8,
    pub delegators: BTreeMap<String, BTreeSet<String>>,
    pub signers: BTreeMap<String, Vec<u8>>,
    pub next_signers: BTreeSet<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub claim_events: BTreeMap<String, Vec<u8>>,
}

pub(crate) fn verify(envelope: &IdEventEnvelope) -> Result<Vec<u8>, IdEventError> {
    let cid = Cid::from_str(&envelope.id)?;
    cid.ensure(&envelope.payload, vec![ED_CODE])?;
    let inception: IdInception = cbor::decode(&envelope.payload)?;

    // Timestamp check
    //
    let valid_from: DateTime<Utc> = VALID_FROM.parse().expect("Invalid date format");
    if inception.timestamp < valid_from.timestamp() {
        return Err(IdEventError::InvalidTimestamp);
    }

    // Validate signer thresholds
    let total_signers = inception.signers.len() as u8;
    if inception.threshold == 0 || total_signers < inception.threshold {
        return Err(IdEventError::ThresholdNotMatch);
    }

    let total_next_signers = inception.next_signers.len() as u8;
    if inception.next_threshold == 0 || total_next_signers < inception.next_threshold {
        return Err(IdEventError::NextThresholdNotMatch);
    }

    // Validate signer key ids and key material
    for (kid, pk) in &inception.signers {
        // Ensure KID is a valid CID and codec matches ED25519
        let parsed = Cid::from_str(kid)?;
        if parsed.codec() != ED_CODE {
            return Err(IdEventError::InvalidSigner(kid.clone()));
        }
        // Ensure KID matches the provided public key bytes
        let expected = Cid::create(ED_CODE, pk)?;
        if expected.to_string() != *kid {
            return Err(IdEventError::InvalidSigner(kid.clone()));
        }
    }

    // Validate next signer ids
    for next_kid in &inception.next_signers {
        let parsed = Cid::from_str(next_kid)
            .map_err(|_| IdEventError::InvalidNextSigner(next_kid.clone()))?;
        if parsed.codec() != ED_CODE {
            return Err(IdEventError::InvalidNextSigner(next_kid.clone()));
        }
    }

    // Validate delegators and their key references
    for (delegation_id, keys) in &inception.delegators {
        let _ = Cid::from_str(delegation_id)
            .map_err(|_| IdEventError::InvalidDelegationId(delegation_id.clone()))?;
        for k in keys {
            let parsed = Cid::from_str(k).map_err(|_| IdEventError::InvalidSigner(k.clone()))?;
            if parsed.codec() != ED_CODE {
                return Err(IdEventError::InvalidSigner(k.clone()));
            }
        }
    }

    // Verify delegator proofs via host and ensure they correspond to declared delegators
    for proof in &envelope.delegator_proofs {
        // Must correspond to a declared delegator id if any are present
        if !inception.delegators.is_empty() && !inception.delegators.contains_key(&proof.id) {
            return Err(IdEventError::InvalidDelegationId(proof.id.clone()));
        }
        let proof_bytes = serde_json::to_vec(proof)?;
        match crate::host::call(&proof_bytes, None) {
            Ok(_) => {}
            Err(_) => return Err(IdEventError::InvalidDelegationId(proof.id.clone())),
        }
    }

    let mut id_state = IdState {
        id: envelope.id.clone(),
        event_id: envelope.id.clone(),
        event_timestamp: Utc
            .timestamp_micros(inception.timestamp)
            .single()
            .ok_or(IdEventError::InvalidTimestamp)?,
        prior_id: inception.prior_id.clone(),
        threshold: inception.threshold,
        next_threshold: inception.next_threshold,
        delegators: inception.delegators,
        signers: inception
            .signers
            .iter()
            .map(|(k, v)| (k.clone(), IdSigner::new(v)))
            .collect(),
        current_signers: inception
            .signers
            .iter()
            .map(|(k, ..)| (k.clone()))
            .collect(),
        next_signers: inception.next_signers.clone(),
        claim_events: inception
            .claim_events
            .iter()
            .map(|(k, v)| (k.clone(), vec![v.clone()]))
            .collect(),
    };

    for (claim_key, claim_event) in &inception.claim_events {
        id_state
            .claim_events
            .insert(claim_key.to_owned(), vec![claim_event.to_owned()]);
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
        let id = Cid::create(ED_CODE, signing_key.as_bytes())
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
            timestamp: Utc::now().timestamp(),
            next_signers: next_signers,
            prior_id: None,
            threshold: 1,
            next_threshold: 1,
            delegators: BTreeMap::new(),
            signers: signers,
            claim_events: BTreeMap::new(),
        };
        let inception_bytes = cbor::encode(&inception);
        let id = Cid::create(CBOR_CODE, inception_bytes.as_slice())
            .unwrap()
            .to_string();
        eprintln!("ID: {}", id.to_string());
        let pinception = IdEventEnvelope {
            id: id.to_string(),
            created_at: Utc::now(),
            payload: inception_bytes,
            proofs: BTreeMap::new(),
            delegator_proofs: vec![],
        };
        let result = verify(&pinception);
        assert!(result.is_ok());
        let result: IdState = cbor::decode(&result.unwrap()).unwrap();
        eprintln!("Result: {:#?}", result);
    }
}
