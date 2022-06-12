use std::collections::HashMap;

use idp2p_common::{
    chrono::Utc,
    cid::Cid,
    multi::{
        hash::Idp2pHash,
        id::{Idp2pCid, Idp2pCodec},
        key::Idp2pKey,
    },
};

use super::microledger::{EventLog, EventLogPayload, IdentityInception, Microledger};
use crate::identity::{
    error::IdentityError,
    models::{ChangeType, IdEvent},
    state::{IdentityState, IdentityStateEventHandler},
    ChangeInput, CreateIdentityInput, IdBehaviour, Identity,
};
pub struct JsonIdentityBehavior;

impl IdBehaviour for JsonIdentityBehavior {
    fn new(&self, input: CreateIdentityInput) -> Result<Identity, IdentityError> {
        let mut inception = IdentityInception {
            timestamp: Utc::now().timestamp(),
            next_key_digest: input.next_key_digest,
            recovery_key_digest: input.recovery_key_digest,
            events: vec![],
        };
        for id_event in input.events {
            match id_event {
                IdEvent::RevokeAssertionKey(_)
                | IdEvent::RevokeAuthenticationKey(_)
                | IdEvent::RevokeAgreementKey(_) => {
                    return Err(IdentityError::Other);
                }
                _ => inception.events.push(id_event.into()),
            }
        }

        let inception_bytes = serde_json::to_vec(&inception)?;
        let cid = Cid::new_cid(Idp2pCodec::Json, &inception_bytes);
        let microledger = Microledger {
            inception: inception_bytes,
            event_logs: vec![],
        };
        let did = Identity {
            id: cid.into(),
            microledger: serde_json::to_vec(&microledger)?,
        };
        Ok(did)
    }

    fn change(&self, did: &mut Identity, input: ChangeInput) -> Result<bool, IdentityError> {
        let state = self.verify(did, None)?;
        let signer_key: Idp2pKey = input.signer_keypair.to_key();

        let payload = EventLogPayload {
            previous: state.last_event_id,
            signer_key: signer_key.to_raw_bytes(),
            next_key_digest: input.next_key_digest,
            timestamp: Utc::now().timestamp(),
            change: input.change,
        };

        let payload_bytes = serde_json::to_vec(&payload)?;
        let proof = input.signer_keypair.sign(&payload_bytes);
        let event_log = EventLog {
            event_id: Idp2pHash::default().digest(&payload_bytes).to_bytes(),
            payload: payload_bytes,
            proof: proof,
        };
        let mut microledger: Microledger = serde_json::from_slice(&did.microledger)?;
        microledger.event_logs.push(event_log);
        did.microledger = serde_json::to_vec(&microledger)?;
        Ok(true)
    }

    fn verify(
        &self,
        identity: &Identity,
        prev: Option<&Identity>,
    ) -> Result<IdentityState, IdentityError> {
        let microledger: Microledger = serde_json::from_slice(&identity.microledger)?;
        let cid = Cid::from_bytes(&identity.id)?;
        cid.ensure(&microledger.inception)?;
        if let Some(prev) = prev {
            is_valid_previous(&microledger, prev)?;
        }
        let inception: IdentityInception = serde_json::from_slice(&microledger.inception)?;
        // Init current state to handle events
        let mut state = IdentityState {
            id: cid.to_bytes(),
            last_event_id: cid.hash().to_bytes(), // First event is the id hash
            next_key_digest: inception.next_key_digest,
            recovery_key_digest: inception.recovery_key_digest,
            assertion_keys: vec![],
            authentication_keys: vec![],
            agreement_keys: vec![],
            proofs: HashMap::new(),
        };
        // Handle initial events
        for event in inception.events {
            state.handle_event(inception.timestamp, event)?;
        }
        for log in microledger.event_logs {
            let payload: EventLogPayload = resolve_payload(&log, &state.last_event_id)?;
            match payload.change {
                ChangeType::Recover(rec_key_digest) => {
                    let signer = state.next_recovery_key(&payload.signer_key)?;
                    signer.verify(&log.payload, &log.proof)?;
                    state.recovery_key_digest = rec_key_digest;
                }
                ChangeType::AddEvents { events } => {
                    let signer = state.next_signer_key(&payload.signer_key)?;
                    signer.verify(&log.payload, &log.proof)?;
                    for event in events {
                        state.handle_event(payload.timestamp, event)?;
                    }
                }
            }
            state.next_key_digest = payload.next_key_digest;
            state.last_event_id = log.event_id;
        }
        Ok(state)
    }
}

fn is_valid_previous(can_ml: &Microledger, prev: &Identity) -> Result<bool, IdentityError> {
    let prev_ml: Microledger = serde_json::from_slice(&prev.microledger)?;
    for (i, log) in prev_ml.event_logs.iter().enumerate() {
        if log.event_id != can_ml.event_logs[i].event_id {
            return Err(IdentityError::InvalidId);
        }
    }
    Ok(true)
}

fn resolve_payload(log: &EventLog, last_event_id: &[u8]) -> Result<EventLogPayload, IdentityError> {
    let payload: EventLogPayload = serde_json::from_slice(&log.payload)?;
    // Get multihash of last_event_id
    let mh = Idp2pHash::from_bytes(&log.event_id)?;
    let hash = Idp2pHash::try_from(mh.code())?;
    // Ensure generated id equals event_id
    hash.ensure(mh, &log.payload)?;
    // Previous event_id should match with last_event_id of state.
    // Because all identity events point previous event.
    // First event points to inception event
    if payload.previous != *last_event_id {
        return Err(IdentityError::InvalidPrevious);
    }
    Ok(payload)
}
