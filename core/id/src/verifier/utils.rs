use alloc::{str::FromStr, string::String};
use chrono::{DateTime, SecondsFormat, TimeZone, Utc};
use ciborium::cbor;
use cid::Cid;
use idp2p_common::{CBOR_CODE, ED_CODE, cid::CidExt, ed25519, error::CommonError};

use crate::types::IdEventReceipt;

use super::{error::IdEventError, signer::IdSigner};

// Wrapper for seconds-since-epoch timestamps
pub(crate) struct Timestamp(pub i64);

impl TryFrom<Timestamp> for String {
    type Error = IdEventError;

    fn try_from(value: Timestamp) -> Result<Self, Self::Error> {
        Ok(Utc
            .timestamp_opt(value.0, 0)
            .single()
            .ok_or(IdEventError::InvalidTimestamp)?
            .to_rfc3339_opts(SecondsFormat::Secs, true))
    }
}

impl TryFrom<Timestamp> for DateTime<Utc> {
    type Error = IdEventError;

    fn try_from(value: Timestamp) -> Result<Self, Self::Error> {
        Utc.timestamp_opt(value.0, 0)
            .single()
            .ok_or(IdEventError::InvalidTimestamp)
    }
}

pub fn verify_delegation_proofs(
    receipt: &IdEventReceipt,
    delegators: &Vec<String>,
) -> Result<(), IdEventError> {
    for delegator in delegators {
        let proof = receipt
            .external_proofs
            .iter()
            .find(|p| p.id == *delegator)
            .ok_or(IdEventError::LackOfMinProofs)?;
        let created_at: DateTime<Utc> = proof
            .created_at
            .parse()
            .map_err(|_| IdEventError::invalid_proof(&proof.id, "Invalid created_at"))?;
        let cid = Cid::try_from(proof.content_id.clone())?;
        cid.ensure(&receipt.payload, vec![CBOR_CODE])?;
        let data = cbor!({
            "id" => proof.id.clone(),
            "purpose" => "delegation",
            "version" => proof.version.clone(),
            "key_id" => proof.key_id.clone(),
            "created_at" => created_at.timestamp(),
            "content_id" => proof.content_id,
        })
        .map_err(|_| CommonError::EncodeError)?;
        let data_bytes = idp2p_common::cbor::encode(&data);
        crate::host::verify_proof(&proof, &data_bytes).map_err(|_| {
            IdEventError::invalid_proof(&proof.id, "Delegated proof verification failed")
        })?;
    }
    Ok(())
}
