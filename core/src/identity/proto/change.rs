use idp2p_common::{
    chrono::Utc,
    multi::{hash::Idp2pHash, key::Idp2pKey},
};
use prost::Message;

use crate::{
    identity::{error::IdentityError, ChangeInput, ChangeType, IdEvent, Identity},
    idp2p_proto::{
        event_log_payload::{Change, IdentityEvents},
        EventLog, EventLogPayload, IdentityEvent, Microledger,
    },
};

use super::verify::verify;

pub fn change(did: &mut Identity, input: ChangeInput) -> Result<(), IdentityError> {
    let state = verify(did, None)?;
    let signer_key: Idp2pKey = input.signer_keypair.to_key();
    let mut payload = EventLogPayload {
        version: 1,
        previous: state.last_event_id,
        signer_key: signer_key.to_bytes(),
        next_key_digest: input.next_key_digest.to_bytes(),
        timestamp: Utc::now().timestamp(),
        change: None,
    };

    match input.change {
        ChangeType::AddEvents(events) => {
            macro_rules! validate_new_key {
                ($ks: ident, $kid: expr) => {{
                    if state.$ks.iter().any(|k| k.id == $kid) {
                        return Err(IdentityError::InvalidCreateKey);
                    }
                }};
            }
            macro_rules! validate_revoke_key {
                ($ks: ident, $kid: expr) => {{
                    if state.$ks.iter().any(|k| k.id == $kid) {
                        return Err(IdentityError::InvalidRevokeKey);
                    }
                }};
            }
            let mut id_events: Vec<IdentityEvent> = vec![];
            for event in events {
                match &event {
                    IdEvent::CreateAssertionKey { id, key: _ } => {
                        validate_new_key!(assertion_keys, *id)
                    }
                    IdEvent::CreateAuthenticationKey { id, key: _ } => {
                        validate_new_key!(authentication_keys, *id)
                    }
                    IdEvent::CreateAgreementKey { id, key: _ } => {
                        validate_new_key!(agreement_keys, *id)
                    }
                    IdEvent::RevokeAssertionKey(kid) => validate_revoke_key!(assertion_keys, *kid),
                    IdEvent::RevokeAuthenticationKey(kid) => {
                        validate_revoke_key!(authentication_keys, *kid)
                    }
                    IdEvent::RevokeAgreementKey(kid) => validate_revoke_key!(agreement_keys, *kid),
                    _ => {}
                }
                id_events.push(event.into());
            }
            payload.change = Some(Change::Events(IdentityEvents { events: id_events }));
        }
        ChangeType::Recover(key_digest) => {
            payload.change = Some(Change::Recover(key_digest.to_bytes()));
        }
    }
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
