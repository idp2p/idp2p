use super::{
    claim::IdClaim,
    error::IdEventError,
    signer::IdSigner,
    utils::{verify_delegation_proofs, verify_proofs},
};
use crate::{
    VALID_FROM, VERSION,
    types::{IdEventReceipt, IdState},
};

use alloc::collections::BTreeSet;
use alloc::{str::FromStr, string::String};
use chrono::{DateTime, SecondsFormat, TimeZone, Utc};
use cid::Cid;
use idp2p_common::{CBOR_CODE, ED_CODE, cbor, cid::CidExt, error::CommonError};
use serde::{Deserialize, Serialize};

macro_rules! ensure {
    ($cond:expr, $error:expr) => {
        if !($cond) {
            return Err($error);
        }
    };
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdInception {
    pub version: String,
    pub patch: Cid,
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub prior_id: Option<String>,
    pub threshold: u8,
    pub next_threshold: u8,
    pub signers: BTreeSet<IdSigner>,
    pub next_signers: BTreeSet<String>,
    #[serde(skip_serializing_if = "BTreeSet::is_empty", default)]
    pub claims: BTreeSet<IdClaim>,
}

pub(crate) fn verify(receipt: &IdEventReceipt) -> Result<IdState, IdEventError> {
    ensure!(receipt.version == VERSION, IdEventError::UnsupportedVersion);
    let id = Cid::from_str(&receipt.id)?;
    id.ensure(&receipt.payload, vec![CBOR_CODE])?;
    let inception: IdInception =
        cbor::decode(&receipt.payload).map_err(|e| CommonError::DecodeError(e.to_string()))?;

    let valid_from: DateTime<Utc> = VALID_FROM
        .parse()
        .map_err(|_| IdEventError::InvalidTimestamp)?;
    let total_signers = inception.signers.len() as u8;
    let total_next_signers = inception.next_signers.len() as u8;
    let total_signatures = receipt.proofs.len() as u8;

    ensure!(
        inception.timestamp > valid_from.timestamp(),
        IdEventError::InvalidTimestamp
    );

    ensure!(
        total_signers >= total_signatures,
        IdEventError::LackOfMinProofs
    );

    ensure!(
        total_signatures >= inception.threshold,
        IdEventError::LackOfMinProofs
    );
    ensure!(
        inception.version == VERSION,
        IdEventError::UnsupportedVersion
    );
    ensure!(inception.threshold >= 1, IdEventError::ThresholdNotMatch);

    // Validate signer key ids and proofs
    verify_proofs(&receipt, inception.signers.clone().into_iter().collect())?;
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

    let filtered_delegators: Vec<String> = inception
        .claims
        .iter()
        .filter(|claim| claim.kind == "/idp2p/event-delegator")
        .map(|claim| claim.id.to_owned())
        .collect();
    verify_delegation_proofs(&receipt, &filtered_delegators)?;
    let timestamp = Utc
        .timestamp_micros(inception.timestamp)
        .single()
        .ok_or(IdEventError::InvalidTimestamp)?
        .to_rfc3339_opts(SecondsFormat::Secs, true);
    let id_state = IdState {
        id: receipt.id.clone(),
        event_id: receipt.id.clone(),
        event_timestamp: timestamp.clone(),
        prior_id: inception.prior_id.clone(),
        threshold: inception.threshold,
        next_threshold: inception.next_threshold,
        signers: inception
            .signers
            .clone()
            .into_iter()
            .map(|s| s.to_state(&timestamp))
            .collect(),
        current_signers: inception
            .signers
            .into_iter()
            .map(|signer| signer.id)
            .collect(),
        next_signers: inception.next_signers.into_iter().collect(),
        claims: inception
            .claims
            .into_iter()
            .map(|s| s.to_state(&timestamp))
            .collect(),
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

        signers.insert(IdSigner {
            id: kid.clone(),
            public_key: pk,
        });
        next_signers.insert(kid);
        let inception = IdInception {
            version: "1.0".into(),
            patch: Cid::default(),
            timestamp: Utc::now().timestamp(),
            prior_id: None,
            threshold: 1,
            next_threshold: 1,
            signers: signers,
            next_signers: next_signers,
            claims: BTreeSet::new(),
        };
        let inception_bytes = cbor::encode(&inception);
        let id = Cid::create(CBOR_CODE, inception_bytes.as_slice())
            .unwrap()
            .to_string();
        eprintln!("ID: {}", id.to_string());
        let pinception = IdEventReceipt {
            id: id.to_string(),
            version: "1.0".into(),
            created_at: Utc::now().to_rfc3339(),
            payload: inception_bytes,
            proofs: Vec::new(),
            external_proofs: Vec::new(),
        };
        let result = verify(&pinception);
        eprintln!("Result: {:#?}", result);
        assert!(result.is_ok());
        let result: IdState = result.unwrap();
        eprintln!("Result: {:#?}", result);
    }
}
