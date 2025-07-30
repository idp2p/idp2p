use std::collections::{BTreeMap, BTreeSet};

use super::{IdEventRule, IdSigner, state::IdState};
use crate::{error::IdInceptionError, types::{PersistedIdEvent, PersistedIdProof}, RELEASE_DATE, VERSION};
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
    pub rotation_rule: IdEventRule,
    pub interaction_rule: IdEventRule,
    pub revocation_rule: IdEventRule,
    pub migration_rule: IdEventRule,
    pub signers: BTreeMap<String, Vec<u8>>,
    pub current_signers: BTreeSet<String>,
    pub next_signers: BTreeSet<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub claim_events: BTreeMap<String, Vec<u8>>,
}

impl TryFrom<&PersistedIdEvent> for IdInception {
    type Error = IdInceptionError;

    fn try_from(value: &PersistedIdEvent) -> Result<Self, Self::Error> {
        let cid = Cid::from_str(&value.id)?;
        cid.ensure(&value.payload)?;
        let inception: IdInception = cbor::decode(&value.payload)?;
        Ok(inception)
    }
}

pub(crate) fn verify(pinception: &PersistedIdEvent) -> Result<Vec<u8>, IdInceptionError> {
    if pinception.version != VERSION {
        return Err(IdInceptionError::UnsupportedVersion);
    }

    let inception: IdInception = (pinception).try_into()?;

    // Timestamp check
    //
    let release_date: DateTime<Utc> = RELEASE_DATE.parse().expect("Invalid date format");
    if inception.timestamp < release_date {
        return Err(IdInceptionError::InvalidTimestamp);
    }

    for proof in &pinception.proofs {
        match proof {
            PersistedIdProof::CurrentKey(proof) => {
                
            },
            PersistedIdProof::DelegateKey(proof) => {
                let _ = crate::host::verify_proof("event_time", &vec![], proof).unwrap();
            }
            _ => todo!(),
        }
    }
    // Inception rule check

    let mut id_state = IdState {
        id: pinception.id.clone(),
        event_id: pinception.id.clone(),
        event_timestamp: inception.timestamp,
        prior_id: inception.prior_id.clone(),
        rotation_rule: inception.rotation_rule.clone(),
        interaction_rule: inception.interaction_rule.clone(),
        revocation_rule: inception.revocation_rule.clone(),
        migration_rule: inception.migration_rule.clone(),
        delegates: BTreeSet::new(),
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
            rotation_rule: vec![],
            interaction_rule: vec![],
            revocation_rule: vec![],
            migration_rule: vec![],
            signers: signers,
            current_signers: BTreeSet::new(),
            claim_events: BTreeMap::new(),
        };
        let inception_bytes = cbor::encode(&inception);
        let id = Cid::create(CBOR_CODE, inception_bytes.as_slice())
            .unwrap()
            .to_string();
        eprintln!("ID: {}", id.to_string());
        let pinception = PersistedIdEvent {
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
