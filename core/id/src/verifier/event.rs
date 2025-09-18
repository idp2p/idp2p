use alloc::collections::BTreeSet;

use super::error::IdEventError;
use crate::{
    VALID_FROM, VERSION,
    types::{ IdEventReceipt, IdState},
    verifier::{
        claim::{IdClaimCreateEvent, IdClaimRevokeEvent},
        signer::IdSigner,
    },
};
use IdEventKind::*;
use alloc::str::FromStr;
use alloc::string::String;
use chrono::{DateTime, SecondsFormat, TimeZone, Utc};
use cid::Cid;
use idp2p_common::{CBOR_CODE, ED_CODE, cbor, cid::CidExt};
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
        new_claims: BTreeSet<IdClaimCreateEvent>,
        revoked_claims: BTreeSet<IdClaimRevokeEvent>,
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
        revealed_signers: BTreeSet<IdSigner>,
        // if the threshold increases
        new_signers: BTreeSet<IdSigner>,
        /// next signer ids
        next_signers: BTreeSet<String>,
    },

    /// Should be signed with signers
    /// All signers should be in the state.next_signers
    /// The total number of signers should be greater than or equal state.next_threshold
    Revocation {
        revealed_signers: BTreeSet<IdSigner>,
    },
    /// Should be signed with signers
    /// All signers should be in the state.next_signers
    /// The total number of signers should be greater than or equal state.next_threshold
    Migration {
        revealed_signers: BTreeSet<IdSigner>,
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
            for event in new_claims {
                state.add_claim(event, &timestamp);
            }
            for event in revoked_claims {
                state.revoke_claim(event, &timestamp)?;
            }
        }
        Rotation {
            threshold,
            next_threshold,
            revealed_signers,
            new_signers,
            next_signers,
        } => {
            let all_signers: BTreeSet<IdSigner> =
                revealed_signers.union(&new_signers).cloned().collect();

            let total_signers = all_signers.len() as u8;
            let total_revealed_signers = revealed_signers.len() as u8;
            let total_next_signers = next_signers.len() as u8;

            ensure!(total_signers >= threshold, IdEventError::ThresholdNotMatch);

            ensure!(
                total_revealed_signers >= state.next_threshold,
                IdEventError::ThresholdNotMatch
            );
            for signer in revealed_signers {
                ensure!(
                    state.next_signers.iter().any(|s| s == &signer.id),
                    IdEventError::ThresholdNotMatch
                );
            }

            ensure!(
                total_next_signers >= next_threshold,
                IdEventError::NextThresholdNotMatch
            );
            for next_kid_str in &next_signers {
                let next_kid = Cid::from_str(next_kid_str)?;
                ensure!(
                    next_kid.codec() == ED_CODE,
                    IdEventError::InvalidNextSigner(next_kid_str.clone())
                );
            }
            receipt.verify_proofs(&all_signers)?;
            state.next_signers = next_signers.into_iter().collect();
            state.threshold = threshold;
            state.next_threshold = next_threshold;
        }
        Revocation { revealed_signers } => {
            ensure!(
                revealed_signers.len() as u8 >= state.next_threshold,
                IdEventError::ThresholdNotMatch
            );
            for signer in &revealed_signers {
                ensure!(
                    state.next_signers.iter().any(|s| s == &signer.id),
                    IdEventError::ThresholdNotMatch
                );
            }
            receipt.verify_proofs(&revealed_signers)?;
            state.revoked = true;
            state.revoked_at = Some(timestamp.clone());
        }
        Migration {
            revealed_signers,
            next_id,
        } => {
            ensure!(
                revealed_signers.len() as u8 >= state.next_threshold,
                IdEventError::ThresholdNotMatch
            );
            for signer in &revealed_signers {
                ensure!(
                    state.next_signers.iter().any(|s| s == &signer.id),
                    IdEventError::ThresholdNotMatch
                );
            }
            receipt.verify_proofs(&revealed_signers)?;
            state.next_id = Some(next_id);
        }
    }

    state.event_id = receipt.id.clone();

    Ok(state)
}
