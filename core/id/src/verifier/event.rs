use alloc::collections::{BTreeMap, BTreeSet};

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
    /// Should be signed with current keys
    Interaction {
        new_claims: BTreeSet<IdClaim>,
        revoked_claims: BTreeSet<IdClaim>,
    },

    /// Should be signed with next keys
    Rotation {
        /// The total number of signers should match the current threshold
        threshold: u8,
        /// The total number of next_signers should match the next_threshold
        next_threshold: u8,
        /// The total number of signers should be greater than or equal state.next_signers and threshold.
        /// The new signers might be added when the threshold increased
        /// All the state.next_signers should be inside this list
        signers: BTreeSet<IdSigner>,
        /// The total number of next_signers should be greater than or equal next_threshold
        next_signers: BTreeSet<String>,
    },

    /// Should be signed with next keys
    Revocation {
        /// Each signer should be exactly match state.next_signers
        signers: BTreeSet<IdSigner>,
    },
    /// Should be signed with next keys
    Migration {
        /// Each signer should be exactly match state.next_signers
        signers: BTreeSet<IdSigner>,
        next_id_hash: String,
    },
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdEvent {
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

    // Timestamp check
    let valid_from: DateTime<Utc> = VALID_FROM.parse().expect("Invalid date format");
    if event.timestamp < valid_from.timestamp() {
        return Err(IdEventError::InvalidTimestamp);
    }
    // Previous event check
    if event.previous != state.event_id {
        return Err(IdEventError::PreviousNotMatch);
    }
    let timestamp = Utc
        .timestamp_micros(event.timestamp)
        .single()
        .ok_or(IdEventError::InvalidTimestamp)?
        .to_rfc3339_opts(SecondsFormat::Secs, true);
    let mut proof_signers = BTreeSet::new();
    match event.body {
        Interaction {
            new_claims,
            revoked_claims,
        } => {
            for claim in new_claims {
                state.claims.push(claim.to_state(&timestamp));
            }
            
            proof_signers = state
                .signers
                .iter()
                .map(|s| IdSigner {
                    id: s.id.clone(),
                    public_key: s.public_key.clone(),
                })
                .collect();
        }
        Rotation {
            threshold,
            next_threshold,
            signers,
            next_signers,
        } => {
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
            proof_signers = signers;
        }
        Revocation { signers } => {
            proof_signers = signers;
        }
        Migration { signers, next_id_hash } => {
            proof_signers = signers;
        }
        _ => {}
    }
    receipt.verify_proofs(&proof_signers)?;
    state.event_id = receipt.id.clone();
    state.signers = proof_signers
        .into_iter()
        .map(|s| s.to_state(&timestamp))
        .collect();
    Ok(state)
}
