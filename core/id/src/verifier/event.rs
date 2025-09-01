use alloc::collections::{BTreeMap, BTreeSet};

use crate::{
    VALID_FROM, VERSION,
    types::{IdEventReceipt, IdState},
    verifier::{claim::IdClaim, delegator::IdDelegator, signer::IdSigner},
};
use super::error::IdEventError;
use IdEventKind::*;
use alloc::str::FromStr;
use alloc::string::String;
use alloc::vec::Vec;
use chrono::{DateTime, Utc};
use cid::Cid;
use idp2p_common::{CBOR_CODE, cbor, cid::CidExt, ed25519};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum IdEventKind {
    /// Should be signed with current keys
    Interaction {
        new_claims: BTreeSet<IdClaim>,
        new_aka: BTreeSet<String>,
        new_delegators: BTreeSet<IdDelegator>,
        revoked_claims: BTreeSet<IdClaim>,
        revoked_aka: BTreeSet<String>,
        revoked_delegators: BTreeSet<IdDelegator>,
    },

    /// Should be signed with next keys
    Rotation {
        /// The total number of signers should match the current threshold
        threshold: Option<u8>,
        /// The total number of signers in state.next_signers should match the min next_threshold 
        next_threshold: Option<u8>,
        signers: BTreeSet<IdSigner>,
        next_signers: BTreeSet<String>,
    },

    /// Should be signed with next keys
    Revocation {
        details: Option<String>,
        /// Each signer should be in state.next_signers
        signers: BTreeMap<String, Vec<u8>>,
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

    match event.body {
        Interaction {
            new_claims,
            new_aka,
            new_delegators,
            revoked_claims,
            revoked_aka,
            revoked_delegators,
        } => {
            // Check threshold
            // Verify signatures
            // Verify proofs
            //

            /*if (pevent.proofs.len() as u8) < state.threshold {
                return Err(IdEventError::LackOfMinProofs);
            }
            for (proof_kid, proof_sig) in pevent.proofs {
                let signer_pk = state
                    .signers
                    .get(&proof_kid)
                    .ok_or_else(|| IdEventError::invalid_proof(&proof_kid, "not_found"))?;

                ed25519::verify(signer_pk, &pevent.payload, &proof_sig)?;
            }

            for (claim_key, claim_event) in claims {
                state.claims.entry(claim_key).or_insert(vec![claim_event]);
            }*/
        }
        Rotation {
            threshold,
            next_threshold,
            signers,
            next_signers,
        } => {
            let total_signers = signers.len() as u8;
            if total_signers < state.next_threshold {
                return Err(IdEventError::NextThresholdNotMatch);
            }
            /*for (proof_kid, proof_sig) in pevent.proofs {
                let signer_pk = signers
                    .get(&proof_kid)
                    .ok_or_else(|| IdEventError::invalid_proof(&proof_kid, "not_found"))?;

                ed25519::verify(&signer_pk, &pevent.payload, &proof_sig)?;
            }*/
            state.next_signers = next_signers.into_iter().collect();
            //state.threshold = threshold;
            //state.next_threshold = next_threshold;
        }
        Revocation { details, signers } => {
            /*for (proof_kid, proof_sig) in pevent.proofs {
                let signer_pk = signers
                    .get(&proof_kid)
                    .ok_or_else(|| IdEventError::invalid_proof(&proof_kid, "not_found"))?;

                ed25519::verify(&signer_pk, &pevent.payload, &proof_sig)?;
            }
            state.next_id = Some(next_id);*/
        }
        _ => {}
    }
    state.event_id = receipt.id.clone();

    Ok(state)
}
