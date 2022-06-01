use chrono::Utc;
use prost::Message;

use crate::{
    identity::{error::IdentityError, ChangeInput, Identity, RecoverInput},
    idp2p_proto::{
        event_log_payload::{Change, IdentityEvents},
        EventLog, EventLogPayload, Microledger,
    },
    multi::{hash::Idp2pHash, key::Idp2pKey},
};

use super::verify::verify;

pub fn add_events(did: &mut Identity, input: ChangeInput) -> Result<(), IdentityError> {
    let state = verify(did)?;
    let signer_key: Idp2pKey = input.signer_keypair.to_key();
    let payload = EventLogPayload {
        version: 1,
        previous: state.last_event_id,
        signer_key: signer_key.to_bytes(),
        next_key_digest: input.next_key_digest.to_bytes(),
        timestamp: Utc::now().timestamp(),
        change: Some(Change::Events(IdentityEvents {
            events: input.events.into(),
        })),
    };
    let payload_bytes = payload.encode_to_vec();
    let proof = input.signer_keypair.sign(&payload_bytes);
    let event_log = EventLog {
        event_id: Idp2pHash::default().digest(&payload_bytes).to_bytes(),
        payload: payload_bytes,
        proof: proof,
    };
    let mut microledger = Microledger::decode(&*did.microledger)?;
    microledger.event_logs.push(event_log);
    did.microledger = microledger.encode_to_vec();
    Ok(())
}

pub fn recover(did: &mut Identity, input: RecoverInput) -> Result<(), IdentityError> {
    let state = verify(did)?;
    let signer_key: Idp2pKey = input.signer_keypair.to_key();
    let payload = EventLogPayload {
        version: 1,
        previous: state.last_event_id,
        signer_key: signer_key.to_bytes(),
        next_key_digest: input.next_key_digest.to_bytes(),
        timestamp: Utc::now().timestamp(),
        change: Some(Change::Recover(input.recovery_key_digest.to_bytes())),
    };
    let payload_bytes = payload.encode_to_vec();
    let proof = input.signer_keypair.sign(&payload_bytes);
    let event_log = EventLog {
        event_id: Idp2pHash::default().digest(&payload_bytes).to_bytes(),
        payload: payload_bytes,
        proof: proof,
    };
    let mut microledger = Microledger::decode(&*did.microledger)?;
    microledger.event_logs.push(event_log);
    did.microledger = microledger.encode_to_vec();
    Ok(())
}
