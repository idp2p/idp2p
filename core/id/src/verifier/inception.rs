use super::{claim::IdClaim, delegator::IdDelegator, signer::IdSigner, error::IdEventError};
use crate::{
    VALID_FROM, VERSION,
    types::{IdEventReceipt, IdState},
};

use alloc::collections::BTreeSet;
use alloc::str::FromStr;
use chrono::{DateTime, SecondsFormat, TimeZone, Utc};
use ciborium::cbor;
use cid::Cid;
use idp2p_common::{CBOR_CODE, ed25519};
use idp2p_common::{ED_CODE, cbor, cid::CidExt, error::CommonError};
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
    #[serde(skip_serializing_if = "BTreeSet::is_empty", default)]
    pub aka: BTreeSet<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub prior_id: Option<String>,
    pub threshold: u8,
    pub next_threshold: u8,
    pub signers: BTreeSet<IdSigner>,
    pub next_signers: BTreeSet<String>,
    #[serde(skip_serializing_if = "BTreeSet::is_empty", default)]
    pub delegators: BTreeSet<IdDelegator>,
    #[serde(skip_serializing_if = "BTreeSet::is_empty", default)]
    pub claims: BTreeSet<IdClaim>,
}

pub(crate) fn verify(receipt: &IdEventReceipt) -> Result<IdState, IdEventError> {
    ensure!(receipt.version == VERSION, IdEventError::UnsupportedVersion);
    let id = Cid::from_str(&receipt.id)?;
    id.ensure(&receipt.payload, vec![CBOR_CODE])?;
    let inception: IdInception =
        cbor::decode(&receipt.payload).map_err(|e| CommonError::DecodeError(e.to_string()))?;

    let valid_from: DateTime<Utc> =
        VALID_FROM.parse().map_err(|_| IdEventError::InvalidTimestamp)?;
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
    ensure!(inception.version == VERSION, IdEventError::UnsupportedVersion);
    ensure!(inception.threshold >= 1, IdEventError::ThresholdNotMatch);

    // Validate signer key ids and proofs
    for signer in &inception.signers {
        let kid = Cid::from_str(&signer.id)?;
        kid.ensure(&signer.public_key, vec![ED_CODE])?;
        let proof = receipt
            .proofs
            .iter()
            .find(|p| p.key_id == signer.id)
            .ok_or(IdEventError::LackOfMinProofs)?;
        let created_at: DateTime<Utc> = proof
            .created_at
            .parse()
            .map_err(|_| IdEventError::invalid_proof(&signer.id, "Invalid created_at"))?;
        // protected header
        let data = cbor!({
            "key_id" => signer.id.clone(),
            "created_at" => created_at.timestamp(),
            "payload" => receipt.payload,
        })
        .map_err(|e| CommonError::EncodeError)?
        .as_bytes()
        .ok_or(CommonError::EncodeError)?
        .to_vec();
        match kid.codec() {
            ED_CODE => ed25519::verify(&signer.public_key, &data, &proof.signature)?,
            _ => {
                return Err(IdEventError::InvalidSigner(
                    "Unsupported key type".to_string(),
                ))
            }
        }
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

    let filtered_delegators: Vec<&String> = inception
        .delegators
        .iter()
        .filter(|delegator| delegator.scope.iter().any(|op| op.contains("inception")))
        .map(|delegator| &delegator.id)
        .collect();
    for delegator in filtered_delegators {
        let proof = receipt
            .delegated_proofs
            .iter()
            .find(|p| p.id == *delegator)
            .ok_or(IdEventError::LackOfMinProofs)?;
        let created_at: DateTime<Utc> = proof
            .created_at
            .parse()
            .map_err(|_| IdEventError::invalid_proof(&proof.id, "Invalid created_at"))?;
        let data = cbor!({
            "id" => proof.id.clone(),
            "key_id" => proof.key_id.clone(),
            "version" => proof.version.clone(),
            "created_at" => created_at.timestamp(),
            "payload" => receipt.payload,
        })
        .map_err(|_| CommonError::EncodeError)?
        .as_bytes()
        .ok_or(CommonError::EncodeError)?
        .to_vec();
        crate::host::verify_proof(&proof, &data)
            .map_err(|_| IdEventError::invalid_proof(&proof.id, "Delegated proof verification failed"))?;
    }
    let timestamp = Utc
        .timestamp_micros(inception.timestamp)
        .single()
        .ok_or(IdEventError::InvalidTimestamp)?
        .to_rfc3339_opts(SecondsFormat::Secs, true);
    let id_state = IdState {
        id: receipt.id.clone(),
        event_id: receipt.id.clone(),
        event_timestamp: timestamp.clone(),
        aka: inception.aka.into_iter().collect(),
        prior_id: inception.prior_id.clone(),
        threshold: inception.threshold,
        next_threshold: inception.next_threshold,
        delegators: inception
            .delegators
            .into_iter()
            .map(|s| s.to_state(&timestamp))
            .collect(),
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
            aka: BTreeSet::new(),
            threshold: 1,
            next_threshold: 1,
            signers: signers,
            next_signers: next_signers,
            delegators: BTreeSet::new(),
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
            delegated_proofs: Vec::new(),
        };
        let result = verify(&pinception);
        eprintln!("Result: {:#?}", result);
        assert!(result.is_ok());
        let result: IdState = result.unwrap();
        eprintln!("Result: {:#?}", result);
    }
}
