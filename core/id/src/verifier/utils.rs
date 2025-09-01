use alloc::str::FromStr;
use chrono::{DateTime, Utc};
use ciborium::cbor;
use cid::Cid;
use idp2p_common::{ED_CODE, cid::CidExt, ed25519, error::CommonError};

use crate::types::IdEventReceipt;

use super::{error::IdEventError, signer::IdSigner};

pub(crate) fn verify_proofs(
    signers: Vec<IdSigner>,
    receipt: IdEventReceipt,
) -> Result<(), IdEventError> {
    // Validate signer key ids and proofs
    for signer in &signers {
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
                ));
            }
        }
    }

    /*ensure!(
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
        .to_rfc3339_opts(SecondsFormat::Secs, true);*/
    Ok(())
}
