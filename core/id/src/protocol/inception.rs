use std::collections::{BTreeMap, BTreeSet};

use super::{ IdSigner, state::IdState};
use crate::{
    VALID_FROM, VERSION,
    error::IdInceptionError,
    types::{IdEventEnvelope, IdProofEnvelope},
};
use alloc::str::FromStr;
use chrono::{DateTime, Utc};
use cid::Cid;
use idp2p_common::{cbor, cid::CidExt};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IdInception {
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub prior_id: Option<String>,
    pub threshold: u8,
    pub next_threshold: u8,
    pub delegates: BTreeSet<String>,
    pub signers: BTreeMap<String, Vec<u8>>,
    pub current_signers: BTreeSet<String>,
    pub next_signers: BTreeSet<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub claim_events: BTreeMap<String, Vec<u8>>,
}

impl TryFrom<&IdEventEnvelope> for IdInception {
    type Error = IdInceptionError;

    fn try_from(value: &IdEventEnvelope) -> Result<Self, Self::Error> {
        if value.version != VERSION {
            return Err(IdInceptionError::UnsupportedVersion);
        }
        let cid = Cid::from_str(&value.id)?;
        cid.ensure(&value.payload)?;
        let inception: IdInception = cbor::decode(&value.payload)?;
        Ok(inception)
    }
}

pub(crate) fn verify(envelope: &IdEventEnvelope) -> Result<Vec<u8>, IdInceptionError> {
    let inception: IdInception = (envelope).try_into()?;

    // Timestamp check
    //
    let valid_from: DateTime<Utc> = VALID_FROM.parse().expect("Invalid date format");
    if inception.timestamp < valid_from {
        return Err(IdInceptionError::InvalidTimestamp);
    }

    for proof in &envelope.proofs {
        let _ = crate::host::verify_proof(&vec![], proof).unwrap();
    }

    let mut id_state = IdState {
        id: envelope.id.clone(),
        event_id: envelope.id.clone(),
        event_timestamp: inception.timestamp,
        prior_id: inception.prior_id.clone(),
        threshold: inception.threshold,
        next_threshold: inception.next_threshold,
        delegates: inception.delegates,
        signers: inception
            .signers
            .iter()
            .map(|(k, v)| (k.clone(), IdSigner::new(v)))
            .collect(),
        current_signers: inception.current_signers.clone(),
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
    let mut result: Vec<u8> = Vec::with_capacity(id_state_bytes.len() + 2);
    result.extend_from_slice(0u16.to_be_bytes().as_slice());
    result.extend_from_slice(&id_state_bytes);
    Ok(result)
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
            timestamp: Utc::now(),
            next_signers: next_signers,
            prior_id: None,
            threshold: 1,
            next_threshold: 1,
            delegates: BTreeSet::new(),
            signers: signers,
            current_signers: BTreeSet::new(),
            claim_events: BTreeMap::new(),
        };
        let inception_bytes = cbor::encode(&inception);
        let id = Cid::create(CBOR_CODE, inception_bytes.as_slice())
            .unwrap()
            .to_string();
        eprintln!("ID: {}", id.to_string());
        let pinception = IdEventEnvelope {
            id: id.to_string(),
            payload: inception_bytes,
            version: VERSION.to_string(),
            proofs: vec![],
        };
        let result = verify(&pinception);
        assert!(result.is_ok());
        let result: IdState = cbor::decode(&result.unwrap()).unwrap();
        eprintln!("Result: {:#?}", result);
    }
}
