use std::collections::HashMap;

use idp2p_common::{
    cid::Cid,
    multi::{
        agreement_key::Idp2pAgreementKey, id::Idp2pCid, key::Idp2pKey, key_digest::Idp2pKeyDigest,
    },
};
use prost::Message;

use crate::{
    identity::{
        error::IdentityError,
        state::{AgreementKeyState, IdentityState, KeyState, ProofState},
        Identity,
    },
    idp2p_proto::{
        event_log_payload::Change, identity_event::EventType, IdentityInception, Microledger,
    },
};

use super::mapper::EventLogResolver;

pub fn verify(
    identity: &Identity,
    prev: Option<&Identity>,
) -> Result<IdentityState, IdentityError> {
    let microledger = Microledger::decode(&*identity.microledger)?;
    if let Some(prev) = prev {}
    let cid = Cid::from_bytes(&identity.id)?;
    cid.ensure(&microledger.inception)?;
    // Decode inception bytes of microledger
    let inception = IdentityInception::decode(&*microledger.inception)?;
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
    for log in &microledger.event_logs {
        let payload = log.try_resolve_payload(&state.last_event_id)?;
        let change = payload.change.ok_or(IdentityError::InvalidProtobuf)?;
        match change {
            Change::Recover(change) => {
                let signer = state.recovery_key_digest.to_next_key(&payload.signer_key)?;
                signer.verify(&log.payload, &log.proof)?;
                state.recovery_key_digest = Idp2pKeyDigest::from_bytes(change)?;
            }
            Change::Events(change) => {
                let signer = state.next_key_digest.to_next_key(&payload.signer_key)?;
                signer.verify(&log.payload, &log.proof)?;
                for event in change.events {
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

fn is_valid_previous(can_ml: Microledger, prev: &Identity) -> Result<bool, IdentityError> {
    let prev_ml = Microledger::decode(&*prev.microledger)?;
    for (i, log) in prev_ml.event_logs.iter().enumerate() {
        if log.event_id != can_ml.event_logs[i].event_id {
            return Err(IdentityError::InvalidId);
        }
    }
    Ok(true)
}

impl IdentityState {
    fn handle_event(&mut self, timestamp: i64, event: EventType) -> Result<(), IdentityError> {
        match event {
            EventType::CreateAssertionKey(key) => {
                let previous_key = self.assertion_keys.last_mut();
                if let Some(previous_key) = previous_key {
                    previous_key.expired_at = Some(timestamp);
                }
                let assertion_method = KeyState {
                    id: key.id,
                    valid_at: timestamp,
                    expired_at: None,
                    key: Idp2pKey::from_bytes(key.value)?,
                };
                self.assertion_keys.push(assertion_method);
            }
            EventType::CreateAuthenticationKey(key) => {
                let previous_key = self.authentication_keys.last_mut();
                if let Some(previous_key) = previous_key {
                    previous_key.expired_at = Some(timestamp);
                }
                let authentication_method = KeyState {
                    id: key.id,
                    valid_at: timestamp,
                    expired_at: None,
                    key: Idp2pKey::from_bytes(key.value)?,
                };
                self.authentication_keys.push(authentication_method);
            }
            EventType::CreateAgreementKey(key) => {
                let previous_key = self.agreement_keys.last_mut();
                if let Some(previous_key) = previous_key {
                    previous_key.expired_at = Some(timestamp);
                }
                let agreement_method = AgreementKeyState {
                    id: key.id,
                    valid_at: timestamp,
                    expired_at: None,
                    key: Idp2pAgreementKey::from_bytes(key.value)?,
                };
                self.agreement_keys.push(agreement_method);
            }
            EventType::RevokeAssertionKey(kid) => {
                let key = self.assertion_keys.iter_mut().find(|k| k.id == kid);
                if let Some(key) = key {
                    key.expired_at = Some(timestamp);
                }
            }
            EventType::RevokeAuthenticationKey(kid) => {
                let key = self.authentication_keys.iter_mut().find(|k| k.id == kid);
                if let Some(key) = key {
                    key.expired_at = Some(timestamp);
                }
            }
            EventType::RevokeAgreementKey(kid) => {
                let key = self.agreement_keys.iter_mut().find(|k| k.id == kid);
                if let Some(key) = key {
                    key.expired_at = Some(timestamp);
                }
            }
            EventType::SetProof(proof) => {
                let entry = self.proofs.get_mut(&proof.key);
                if let Some(entry) = entry {
                    entry.expired_at = Some(timestamp);
                }
                self.proofs.insert(
                    proof.key,
                    ProofState {
                        valid_at: timestamp,
                        expired_at: None,
                        value: proof.value,
                    },
                );
            }
        }
        Ok(())
    }
}
