use alloc::collections::BTreeSet;

use super::error::IdEventError;
use crate::{
    VALID_FROM, VERSION,
    types::{IdEventReceipt, IdState},
    verifier::{claim::IdClaim, signer::IdSigner},
};
use IdEventKind::*;
use alloc::str::FromStr;
use alloc::string::String;
use chrono::{DateTime, SecondsFormat, TimeZone, Utc};
use cid::Cid;
use idp2p_common::{CBOR_CODE, cbor, cid::CidExt};
use serde::{Deserialize, Serialize};

macro_rules! ensure {
    ($cond:expr, $error:expr) => {
        if !($cond) {
            return Err($error);
        }
    };
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum IdEventKind {
    /// Should be signed with state.current_signers
    /// The total number of signers should match the state.threshold
    Interaction {
        new_claims: BTreeSet<IdClaim>,
        revoked_claims: BTreeSet<IdClaim>,
    },

    /// Should be signed with signers and new_signers
    /// The total number of signers + new_signers should match the current threshold
    /// The total number of signers should be greater than or equal state.next_threshold
    /// The total number of next_signers should be equal next_threshold
    /// All signers should be in the state.next_signers
    Rotation {
        /// new threshold
        threshold: u8,
        /// new next_threshold
        next_threshold: u8,
        /// signers in state.next_signers
        signers: BTreeSet<IdSigner>,
        // if the threshold increases
        new_signers: BTreeSet<IdSigner>,
        /// next signer ids
        next_signers: BTreeSet<String>,
    },

    /// Should be signed with signers
    /// All signers should be in the state.next_signers
    /// The total number of signers should be greater than or equal state.next_threshold
    Revocation { signers: BTreeSet<IdSigner> },
    /// Should be signed with signers
    /// All signers should be in the state.next_signers
    /// The total number of signers should be greater than or equal state.next_threshold
    Migration {
        signers: BTreeSet<IdSigner>,
        next_id: String,
    },
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdEvent {
    pub version: String,
    /// Timestamp of event
    pub timestamp: i64,

    // The compoenent
    pub component: Cid,

    /// Previous event id
    pub previous: String,

    /// Event body
    pub body: IdEventKind,
}

pub(crate) fn verify(
    receipt: &IdEventReceipt,
    state: &mut IdState,
) -> Result<IdState, IdEventError> {
    let mut state = state.to_owned();
    let cid = Cid::from_str(&receipt.id)?;
    cid.ensure(&receipt.payload, vec![CBOR_CODE])?;
    let event: IdEvent = cbor::decode(&receipt.payload)?;

    ensure!(event.version == VERSION, IdEventError::UnsupportedVersion);

    // Timestamp check
    let valid_from: DateTime<Utc> = VALID_FROM.parse().expect("Invalid date format");

    ensure!(
        event.timestamp >= valid_from.timestamp(),
        IdEventError::InvalidTimestamp
    );
    // Previous event check
    ensure!(
        event.previous == state.event_id,
        IdEventError::PreviousNotMatch
    );

    let timestamp = Utc
        .timestamp_micros(event.timestamp)
        .single()
        .ok_or(IdEventError::InvalidTimestamp)?
        .to_rfc3339_opts(SecondsFormat::Secs, true);

    match event.body {
        Interaction {
            new_claims,
            revoked_claims,
        } => {
            let proof_signers = state
                .signers
                .iter()
                .map(|s| IdSigner {
                    id: s.id.clone(),
                    public_key: s.public_key.clone(),
                })
                .collect();
            receipt.verify_proofs(&proof_signers)?;
            for claim in new_claims {
                state.claims.push(claim.to_state(&timestamp));
            }
        }
        Rotation {
            threshold,
            next_threshold,
            signers,
            next_signers,
            new_signers,
        } => {
            receipt.verify_proofs(&signers)?;
            let total_signers = signers.len() as u8;
            let total_next_signers = next_signers.len() as u8;
            ensure!(total_signers >= threshold, IdEventError::ThresholdNotMatch);
            ensure!(
                total_signers >= state.next_threshold,
                IdEventError::NextThresholdNotMatch
            );
            for signer_id in state.next_signers {
                ensure!(
                    signers.iter().any(|s| s.id == signer_id),
                    IdEventError::ThresholdNotMatch
                );
            }
            ensure!(
                total_next_signers >= next_threshold,
                IdEventError::NextThresholdNotMatch
            );
            state.next_signers = next_signers.into_iter().collect();
            state.threshold = threshold;
            state.next_threshold = next_threshold;
        }
        Revocation { signers } => {
             receipt.verify_proofs(&signers)?;
        }
        Migration { signers, next_id } => {
            receipt.verify_proofs(&signers)?;
            state.next_id = Some(next_id);
        }
    }

    state.event_id = receipt.id.clone();
    /*state.signers = proof_signers
        .into_iter()
        .map(|s| s.to_state(&timestamp))
        .collect();*/
    Ok(state)
}
