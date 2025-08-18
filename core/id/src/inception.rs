use alloc::collections::BTreeSet;

use crate::types::{IdClaimEvent, IdDelegator, IdEventEnvelope, IdSigner, IdState};
use crate::{VALID_FROM, error::IdEventError};
use alloc::str::FromStr;
use chrono::{DateTime, SecondsFormat, TimeZone, Utc};
use cid::Cid;
use idp2p_common::{ED_CODE, cbor, cid::CidExt};
use serde::{Deserialize, Serialize};

macro_rules! ensure {
    ($cond:expr, $error:expr) => {
        if !($cond) {
            return Err($error);
        }
    };
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IdInception {
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub prior_id: Option<String>,
    pub threshold: u8,
    pub next_threshold: u8,
    pub signers: BTreeSet<IdSigner>,
    pub next_signers: BTreeSet<String>,
    #[serde(skip_serializing_if = "BTreeSet::is_empty", default)]
    pub delegators: BTreeSet<IdDelegator>,
    #[serde(skip_serializing_if = "BTreeSet::is_empty", default)]
    pub claim_events: BTreeSet<IdClaimEvent>,
}

pub(crate) fn verify(envelope: &IdEventEnvelope) -> Result<IdState, IdEventError> {
    let id = Cid::from_str(&envelope.id)?;
    id.ensure(&envelope.payload, vec![ED_CODE])?;
    let inception: IdInception = cbor::decode(&envelope.payload)?;

    // Timestamp check
    //
    let valid_from: DateTime<Utc> = VALID_FROM.parse().expect("Invalid date format");
    let total_signers = inception.signers.len() as u8;
    let total_next_signers = inception.next_signers.len() as u8;
    let total_signatures = envelope.proofs.len() as u8;

    ensure!(
        inception.timestamp > valid_from.timestamp(),
        IdEventError::InvalidTimestamp
    );

    ensure!(
        total_signers >= total_signatures,
        IdEventError::ThresholdNotMatch
    );

    ensure!(
        total_signatures >= inception.threshold,
        IdEventError::ThresholdNotMatch
    );

    ensure!(inception.threshold >= 1, IdEventError::ThresholdNotMatch);

    // Validate signer key ids and proofs
    for signer in &inception.signers {
        let kid = Cid::from_str(&signer.id)?;
        kid.ensure(&signer.public_key, vec![ED_CODE])?;
        let proof = envelope
            .proofs
            .iter()
            .find(|p| p.key_id == signer.id)
            .ok_or(IdEventError::ThresholdNotMatch)?;
        idp2p_common::ed25519::verify(&signer.public_key, &envelope.payload, &proof.signature)?;
    }

    ensure!(
        total_next_signers >= inception.next_threshold,
        IdEventError::NextThresholdNotMatch
    );

    // Validate next signer ids
    for next_kid_str in &inception.next_signers {
        let next_kid = Cid::from_str(next_kid_str)?;
        ensure!(
            next_kid.codec() == ED_CODE,
            IdEventError::InvalidNextSigner(next_kid_str.clone())
        );
    }

    // Validate delegators and proofs

    let filtered_keys: Vec<&String> = inception
        .delegators
        .iter()
        .filter(|delegator| {
            delegator
                .restrictions
                .iter()
                .any(|op| op.contains("inception"))
        })
        .map(|delegator| &delegator.id)
        .collect();
    for delegator in filtered_keys {
        let proof = envelope
            .delegated_proofs
            .iter()
            .find(|p| p.id == *delegator)
            .ok_or(IdEventError::NextThresholdNotMatch)?;
    }

    /* // Verify delegator proofs via host and ensure they correspond to declared delegators
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
     }*/

    let id_state = IdState {
        id: envelope.id.clone(),
        event_id: envelope.id.clone(),
        event_timestamp: Utc
            .timestamp_micros(inception.timestamp)
            .single()
            .ok_or(IdEventError::InvalidTimestamp)?
            .to_rfc3339_opts(SecondsFormat::Secs, true),
        prior_id: inception.prior_id.clone(),
        threshold: inception.threshold,
        next_threshold: inception.next_threshold,
        delegators: inception.delegators.into_iter().collect(),
        signers: inception.signers.clone().into_iter().collect(),
        current_signers: inception
            .signers
            .into_iter()
            .map(|signer| signer.id)
            .collect(),
        next_signers: inception.next_signers.into_iter().collect(),
        claim_events: inception.claim_events.into_iter().collect(),
    };

    Ok(id_state)
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
        let mut signers = BTreeSet::new();
        let mut next_signers = BTreeSet::new();

        signers.insert(IdSigner::new(&kid, &pk));
        next_signers.insert(kid);
        let inception = IdInception {
            timestamp: Utc::now().timestamp(),
            next_signers: next_signers,
            prior_id: None,
            threshold: 1,
            next_threshold: 1,
            delegators: BTreeSet::new(),
            signers: signers,
            claim_events: BTreeSet::new(),
        };
        let inception_bytes = cbor::encode(&inception);
        let id = Cid::create(CBOR_CODE, inception_bytes.as_slice())
            .unwrap()
            .to_string();
        eprintln!("ID: {}", id.to_string());
        let pinception = IdEventEnvelope {
            id: id.to_string(),
            version: "1.0".into(),
            payload: inception_bytes,
            proofs: Vec::new(),
            delegated_proofs: Vec::new(),
        };
        let result = verify(&pinception);
        assert!(result.is_ok());
        let result: IdState = result.unwrap();
        eprintln!("Result: {:#?}", result);
    }
}
