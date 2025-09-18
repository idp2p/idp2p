use super::{
    claim::IdClaimCreateEvent,
    error::IdEventError,
    signer::IdSigner,
    utils::{verify_delegation_proofs, verify_proofs},
};
use crate::{
    types::{IdEventReceipt, IdState}, VALID_FROM, VERSION
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
    pub claims: BTreeSet<IdClaimCreateEvent>,
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

    let timestamp = Utc
        .timestamp_micros(inception.timestamp)
        .single()
        .ok_or(IdEventError::InvalidTimestamp)?
        .to_rfc3339_opts(SecondsFormat::Secs, true);
    receipt.verify_proofs(&inception.signers)?;
    let mut id_state = IdState {
        id: receipt.id.clone(),
        event_id: receipt.id.clone(),
        event_timestamp: timestamp.clone(),
        prior_id: inception.prior_id.clone(),
        next_id: None,
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
        claims: vec![],
        revoked: false,
        revoked_at: None,
    };
    for event in inception.claims {
        id_state.add_claim(event, &timestamp);
    }
    Ok(id_state)
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::{ed25519::signature::SignerMut, SigningKey, VerifyingKey};
    use idp2p_common::{CBOR_CODE, ED_CODE};
    use rand::rngs::OsRng;
    use crate::types::IdProof;

    use super::*;

    fn create_signer() -> (String, VerifyingKey, SigningKey) {
        let mut csprng = OsRng;
        let signing_key: SigningKey = SigningKey::generate(&mut csprng);
         let verifying_key = signing_key.verifying_key();
    
        let id = Cid::create(ED_CODE, verifying_key.as_bytes())
            .unwrap()
            .to_string();
        (id, verifying_key, signing_key)
    }

    #[test]
    fn test_verify_inception() {
        let signer1 = create_signer();
        let signer2 = create_signer();
        let next_signer1 = create_signer();
        let next_signer2 = create_signer();

        let mut signers = BTreeSet::new();
        let mut next_signers = BTreeSet::new();

        signers.insert(IdSigner {
            id: signer1.0.clone(),
            public_key: signer1.1.as_bytes().to_vec(),
        });
        signers.insert(IdSigner {
            id: signer2.0.clone(),
            public_key: signer2.1.as_bytes().to_vec(),
        });

        next_signers.insert(next_signer1.0);
        next_signers.insert(next_signer2.0);
        let inception = IdInception {
            version: "1.0".into(),
            patch: Cid::default(),
            timestamp: Utc::now().timestamp(),
            prior_id: None,
            threshold: 1,
            next_threshold: 1,
            signers: signers.clone(),
            next_signers: next_signers,
            claims: BTreeSet::new(),
        };
        let inception_bytes = cbor::encode(&inception);
        let created_at = Utc::now();
       
        let proof1 = IdProof {
            key_id: signer1.0.clone(),
            created_at: created_at.clone().to_rfc3339(),
            signature: signer1.clone().2.sign(&inception_bytes).to_vec(),
        };
         let proof2 = IdProof {
            key_id: signer2.0.clone(),
            created_at: created_at.clone().to_rfc3339(),
            signature: signer2.clone().2.sign(&inception_bytes).to_vec(),
        };
        let id = Cid::create(CBOR_CODE, inception_bytes.as_slice())
            .unwrap()
            .to_string();
        eprintln!("ID: {}", id.to_string());
        let pinception = IdEventReceipt {
            id: id.to_string(),
            version: "1.0".into(),
            created_at: Utc::now().to_rfc3339(),
            payload: inception_bytes,
            proofs: vec![proof1, proof2],
            external_proofs: Vec::new(),
        };
        let result = verify(&pinception);
        eprintln!("Result: {:#?}", result);
        assert!(result.is_ok());
        let result: String = serde_json::to_string_pretty(&result.unwrap()).unwrap();
        eprintln!("Result: {result}");
    }
}
