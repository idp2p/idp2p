use std::{
    collections::{BTreeMap, BTreeSet},
    env,
};

use crate::{
    VALID_FROM, VERSION,
    error::IdEventError,
    model::{envelope::IdEventEnvelope, state::IdState},
};
use IdEventKind::*;
use alloc::str::FromStr;
use alloc::string::String;
use alloc::vec::Vec;
use chrono::{DateTime, Utc};
use cid::Cid;
use idp2p_common::{cbor, cid::CidExt, ed25519, CBOR_CODE};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum IdEventKind {
    /// Should be signed with current keys
    Interaction {
        claim_events: BTreeMap<String, Vec<u8>>
    },

    /// Should be signed with next keys
    Rotation {
        threshold: Option<u8>,
        next_threshold: Option<u8>,
        aka: BTreeSet<String>,
        /// The number of signers in state.next_signers should match the min next_threshold
        /// The totat number of signers should match the current threshold
        signers: BTreeMap<String, Vec<u8>>,
        next_signers: BTreeSet<String>,
    },

    /// Should be signed with next keys
    Revocation {
        details: Option<String>,
        /// Each signer should be in state.next_signers 
        signers: BTreeMap<String, Vec<u8>>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IdEvent {
    /// Timestamp of event
    pub timestamp: i64,

    // The compoenent
    pub component: Vec<u8>,

    /// Previous event id
    pub previous: String,

    /// Event body
    pub body: IdEventKind,
}

pub(crate) fn verify(
    envelope: &IdEventEnvelope,
    state: &mut IdState,
) -> Result<Vec<u8>, IdEventError> {
    let cid = Cid::from_str(&envelope.id)?;
    cid.ensure(&envelope.payload, vec![CBOR_CODE])?;
    let event: IdEvent = cbor::decode(&envelope.payload)?;

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
        Interaction { claim_events } => {
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
            state.next_signers = next_signers;
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
    state.event_id = envelope.id.clone();
    let id_state_bytes = cbor::encode(&state);

    Ok(id_state_bytes)
}
