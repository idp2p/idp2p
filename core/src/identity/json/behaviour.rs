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
            inception: inception,
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
            payload: payload,
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
        let microledger = serde_json::from_slice(&identity.microledger)?;
        if let Some(prev) = prev {
            is_valid_previous(&microledger, prev)?;
        }
        let cid = Cid::from_bytes(&identity.id)?;
        cid.ensure(&serde_json::to_vec(&microledger.inception)?)?;

        // Init current state to handle events
        let mut state = IdentityState {
            id: cid.to_bytes(),
            last_event_id: cid.hash().to_bytes(), // First event is the id hash
            next_key_digest: microledger.inception.next_key_digest,
            recovery_key_digest: microledger.inception.recovery_key_digest,
            assertion_keys: vec![],
            authentication_keys: vec![],
            agreement_keys: vec![],
            proofs: HashMap::new(),
        };
        // Handle initial events
        for event in microledger.inception.events {
            state.handle_event(microledger.inception.timestamp, event)?;
        }
        for log in &microledger.event_logs {
            ensure_payload(log, &state.last_event_id)?;
            match &log.payload.change {
                ChangeType::Recover(change) => {
                    let signer = state
                        .recovery_key_digest
                        .to_next_key(&log.payload.signer_key)?;
                    signer.verify(&serde_json::to_vec(&log.payload)?, &log.proof)?;
                    state.recovery_key_digest = change.to_owned();
                }
                ChangeType::AddEvents { events } => {
                    let signer = state.next_key_digest.to_next_key(&log.payload.signer_key)?;
                    signer.verify(&serde_json::to_vec(&log.payload)?, &log.proof)?;
                    for event in events {
                        state.handle_event(log.payload.timestamp, event.to_owned())?;
                    }
                }
            }
            state.next_key_digest = log.payload.next_key_digest.to_owned();
            state.last_event_id = log.event_id.clone();
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

fn ensure_payload(log: &EventLog, last_event_id: &[u8]) -> Result<(), IdentityError> {
    // Get multihash of last_event_id
    let mh = Idp2pHash::from_bytes(&log.event_id)?;
    let hash = Idp2pHash::try_from(mh.code())?;
    // Ensure generated id equals event_id
    hash.ensure(mh, &serde_json::to_vec(&log.payload)?)?;
    // Previous event_id should match with last_event_id of state.
    // Because all identity events point previous event.
    // First event points to inception event
    if log.payload.previous != *last_event_id {
        return Err(IdentityError::InvalidPrevious);
    }
    Ok(())
}