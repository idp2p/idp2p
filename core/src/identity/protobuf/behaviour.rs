use std::collections::HashMap;

use idp2p_common::{
    chrono::Utc,
    cid::Cid,
    multi::{
        hash::Idp2pHash,
        id::{Idp2pCid, Idp2pCodec},
        key::Idp2pKey,
        key_digest::Idp2pKeyDigest,
    },
};
use prost::Message;

use crate::{
    identity::{
        error::IdentityError,
        models::{ChangeType, IdEvent},
        state::{IdentityState, IdentityStateEventHandler},
        ChangeInput, CreateIdentityInput, IdBehaviour, Identity,
    },
    idp2p_proto,
};

pub struct ProtoIdentityBehavior;

impl IdBehaviour for ProtoIdentityBehavior {
    fn new(&self, input: CreateIdentityInput) -> Result<Identity, IdentityError> {
        let mut inception = idp2p_proto::IdentityInception {
            timestamp: Utc::now().timestamp(),
            next_key_digest: input.next_key_digest.to_bytes(),
            recovery_key_digest: input.recovery_key_digest.to_bytes(),
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

        let inception_bytes = inception.encode_to_vec();
        let cid = Cid::new_cid(Idp2pCodec::Protobuf, &inception_bytes);
        let microledger = idp2p_proto::Microledger {
            inception: inception_bytes,
            event_logs: vec![],
        };
        let did = Identity {
            id: cid.into(),
            microledger: microledger.encode_to_vec(),
        };
        Ok(did)
    }

    fn change(&self, did: &mut Identity, input: ChangeInput) -> Result<bool, IdentityError> {
        use idp2p_proto::event_log_payload::{Change, IdentityEvents};
        let state = self.verify(did, None)?;
        let signer_key: Idp2pKey = input.signer_keypair.to_key();
        let mut payload = idp2p_proto::EventLogPayload {
            previous: state.last_event_id,
            signer_key: signer_key.to_raw_bytes(),
            next_key_digest: input.next_key_digest.to_bytes(),
            timestamp: Utc::now().timestamp(),
            change: None,
        };

        match input.change {
            ChangeType::AddEvents{events} => {
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
                let mut id_events: Vec<idp2p_proto::IdentityEvent> = vec![];
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
                        IdEvent::RevokeAssertionKey(kid) => {
                            validate_revoke_key!(assertion_keys, *kid)
                        }
                        IdEvent::RevokeAuthenticationKey(kid) => {
                            validate_revoke_key!(authentication_keys, *kid)
                        }
                        IdEvent::RevokeAgreementKey(kid) => {
                            validate_revoke_key!(agreement_keys, *kid)
                        }
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
        let event_log = idp2p_proto::EventLog {
            event_id: Idp2pHash::default().digest(&payload_bytes).to_bytes(),
            payload: payload_bytes,
            proof: proof,
        };
        let mut microledger = idp2p_proto::Microledger::decode(&*did.microledger)?;
        microledger.event_logs.push(event_log);
        did.microledger = microledger.encode_to_vec();
        Ok(true)
    }

    fn verify(
        &self,
        identity: &Identity,
        prev: Option<&Identity>,
    ) -> Result<IdentityState, IdentityError> {
        let microledger = idp2p_proto::Microledger::decode(&*identity.microledger)?;
        if let Some(prev) = prev {
            is_valid_previous(&microledger, prev)?;
        }
        let cid = Cid::from_bytes(&identity.id)?;
        cid.ensure(&microledger.inception)?;
        // Decode inception bytes of microledger
        let inception = idp2p_proto::IdentityInception::decode(&*microledger.inception)?;
        
        // Init current state to handle events
        let mut state = IdentityState {
            id: cid.to_bytes(),
            last_event_id: cid.hash().to_bytes(), // First event is the id hash
            next_key_digest: Idp2pKeyDigest::from_bytes(&inception.next_key_digest)?,
            recovery_key_digest: Idp2pKeyDigest::from_bytes(&inception.recovery_key_digest)?,
            assertion_keys: vec![],
            authentication_keys: vec![],
            agreement_keys: vec![],
            proofs: HashMap::new(),
        };
        // Handle initial events
        for event in inception.events {
            let event = event.event_type.ok_or(IdentityError::InvalidProtobuf)?;
            state.handle_event(inception.timestamp, event)?;
        }
        use idp2p_proto::event_log_payload::Change;
        for log in microledger.event_logs {
            let payload = log.try_resolve_payload(&state.last_event_id)?;
            let change = payload.change.ok_or(IdentityError::InvalidProtobuf)?;
            match change {
                Change::Recover(change) => {
                    let signer = state.recovery_key_digest.to_next_key(&payload.signer_key)?;
                    signer.verify(&log.payload, &log.proof)?;
                    state.recovery_key_digest = Idp2pKeyDigest::from_bytes(change)?;
                }
                Change::Events(events) => {
                    let signer = state.next_key_digest.to_next_key(&payload.signer_key)?;
                    signer.verify(&log.payload, &log.proof)?;
                    for event in events.events {
                        let event = event.event_type.ok_or(IdentityError::InvalidProtobuf)?;
                        state.handle_event(payload.timestamp, event)?;
                    }
                }
            }
            state.next_key_digest = Idp2pKeyDigest::from_bytes(payload.next_key_digest)?;
            state.last_event_id = log.event_id.clone();
        }
        Ok(state)
    }
}

use idp2p_proto::Microledger;

use super::mapper::EventLogResolver;
fn is_valid_previous(can_ml: &Microledger, prev: &Identity) -> Result<bool, IdentityError> {
    let prev_ml = Microledger::decode(&*prev.microledger)?;
    for (i, log) in prev_ml.event_logs.iter().enumerate() {
        if log.event_id != can_ml.event_logs[i].event_id {
            return Err(IdentityError::InvalidId);
        }
    }
    Ok(true)
}
