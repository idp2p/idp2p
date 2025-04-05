use std::collections::{BTreeMap, BTreeSet};

use alloc::str::FromStr;
use alloc::string::String;
use alloc::vec::Vec;

use crate::{error::IdEventError, state::IdState, TIMESTAMP, VERSION};
use IdEventKind::*;
use idp2p_common::{cbor, ed25519, identifier::Identifier, wasmsg::Wasmsg};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IdRotation {
    pub threshold: u8,
    pub next_threshold: u8,
    pub signers: BTreeMap<String, Vec<u8>>,
    pub next_signers: BTreeSet<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum IdEventKind {
    /// Should be signed with current keys
    Interaction(BTreeMap<String, Vec<u8>>),

    /// Should be signed with next keys
    Rotation(IdRotation),

    /// Should be signed with next keys
    Migration {
        next_id: String,
        signers: BTreeMap<String, Vec<u8>>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IdEvent {
    /// Timestamp of event
    pub timestamp: i64,

    /// Previous event id
    pub previous: String,

    /// Event payload
    pub payload: IdEventKind,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersistedIdEvent {
    id: String,
    payload: Vec<u8>,
    proofs: BTreeMap<String, Vec<u8>>,
}

impl TryFrom<&PersistedIdEvent> for IdEvent {
    type Error = IdEventError;

    fn try_from(value: &PersistedIdEvent) -> Result<Self, Self::Error> {
        let id = Identifier::from_str(value.id.as_str())
            .map_err(|e| IdEventError::InvalidEventId(e.to_string()))?;
        id.ensure(&value.payload)
            .map_err(|e| IdEventError::InvalidEventId(e.to_string()))?;

        if id.kind != "event" {
            return Err(IdEventError::InvalidEventId(id.to_string()));
        }

        let event: IdEvent =
            cbor::decode(&value.payload).map_err(|_| IdEventError::InvalidPayload)?;
        Ok(event)
    }
}

pub(crate) fn verify(state: &[u8], payload: &[u8]) -> Result<Vec<u8>, IdEventError> {
    let ver = Wasmsg::from_bytes(&payload)?;
    if ver != VERSION {
        return Err(IdEventError::UnsupportedVersion);
    }
    let mut state: IdState = cbor::decode(state)?;
    let pevent: PersistedIdEvent = cbor::decode(payload)?;
    let event: IdEvent = (&pevent).try_into()?;

    // Timestamp check
    if event.timestamp < TIMESTAMP {
        return Err(IdEventError::InvalidTimestamp);
    }

    // Previous event check
    if event.previous != state.event_id {
        return Err(IdEventError::PreviousNotMatch);
    }

    match event.payload {
        Interaction(claims) => {
            if (pevent.proofs.len() as u8) < state.threshold {
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
            }
        }
        Rotation(id_rotation) => {
            let total_signers = id_rotation.signers.len() as u8;
            if total_signers < state.next_threshold {
                return Err(IdEventError::NextThresholdNotMatch);
            }
            for (proof_kid, proof_sig) in pevent.proofs {
                let signer_pk = id_rotation
                    .signers
                    .get(&proof_kid)
                    .ok_or_else(|| IdEventError::invalid_proof(&proof_kid, "not_found"))?;

                ed25519::verify(&signer_pk, &pevent.payload, &proof_sig)?;
            }
            state.next_signers = id_rotation.next_signers;
            state.threshold = id_rotation.threshold;
            state.next_threshold = id_rotation.next_threshold;
        }
        Migration { next_id, signers } => {
            for (proof_kid, proof_sig) in pevent.proofs {
                let signer_pk = signers
                    .get(&proof_kid)
                    .ok_or_else(|| IdEventError::invalid_proof(&proof_kid, "not_found"))?;

                ed25519::verify(&signer_pk, &pevent.payload, &proof_sig)?;
            }
            state.next_id = Some(next_id);
        }
    }
    state.event_id = pevent.id.clone();
    let id_state_bytes = cbor::encode(&state);

    Ok(id_state_bytes)
}
