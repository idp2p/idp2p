use std::collections::HashMap;

use crate::{
    identity::{
        error::IdentityError,
        state::{IdentityState, KeyState, AgreementKeyState},
        ChangeInput, CreateIdentityInput, IdentityBehaviour, RecoverInput,
    },
    idp2p_proto::{
        event_log_payload::{Change, IdentityEvents}, identity_event::EventType, EventLog, EventLogPayload, Identity,
        IdentityInception, Microledger,
    },
    multi::{
        id::{Idp2pCid, Idp2pCodec},
        key::Idp2pKey,
        key_digest::Idp2pKeyDigest, hash::Idp2pHash, agreement_key::Idp2pAgreementKey,
    },
};
use chrono::Utc;
use cid::Cid;
use prost::Message;

use super::mapper::EventLogResolver;

impl IdentityBehaviour for Identity {
    fn new(input: CreateIdentityInput) -> Result<crate::identity::Identity, IdentityError> {
        let mut inception = IdentityInception {
            version: 1,
            timestamp: Utc::now().timestamp(),
            next_key_digest: input.next_key_digest.to_bytes(),
            recovery_key_digest: input.recovery_key_digest.to_bytes(),
            events: vec![],
        };
        inception.events = input.events.into();
        let inception_bytes = inception.encode_to_vec();
        let cid = Cid::new_cid(Idp2pCodec::Protobuf, &inception_bytes);
        let microledger = Microledger {
            inception: inception_bytes,
            event_logs: vec![],
        };
        let did = crate::identity::Identity {
            id: cid.into(),
            microledger: microledger.encode_to_vec(),
        };
        Ok(did)
    }

    fn change(&mut self, input: ChangeInput) -> Result<(), IdentityError> {
        let state = self.verify()?;
        let microledger = self
            .microledger
            .as_mut()
            .ok_or(IdentityError::InvalidProtobuf)?;
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
        microledger.event_logs.push(event_log);
        Ok(())
    }

    fn recover(&mut self, input: RecoverInput) -> Result<(), IdentityError> {
        let state = self.verify()?;
        let microledger = self
            .microledger
            .as_mut()
            .ok_or(IdentityError::InvalidProtobuf)?;
        let signer_key: Idp2pKey = input.signer_keypair.to_key();
        let payload = EventLogPayload {
            version: 1,
            previous: state.last_event_id,
            signer_key: signer_key.to_bytes(),
            next_key_digest: input.next_key_digest.to_bytes(),
            timestamp: Utc::now().timestamp(),
            change: Some(Change::Recover(IdentityEvents {
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
        microledger.event_logs.push(event_log);
        Ok(())
    }

    fn verify(&self) -> Result<IdentityState, IdentityError> {
        let microledger = self
            .microledger
            .as_ref()
            .ok_or(IdentityError::InvalidProtobuf)?;
        let cid: Cid = self.id.to_vec().try_into()?;
        if cid.to_bytes() != self.id {
            return Err(IdentityError::InvalidId);
        }
        // Get inception of microledger
        let inception = IdentityInception::decode(&*microledger.inception)?;
        // Init current state to handle events
        let mut state = IdentityState {
            id: self.id.clone(),
            next_key_digest: Idp2pKeyDigest::from_bytes(&inception.next_key_digest)?,
            recovery_key_digest: Idp2pKeyDigest::from_bytes(&inception.recovery_key_digest)?,
            assertion_keys: vec![],
            authentication_keys: vec![],
            agreement_keys: vec![],
            proofs: HashMap::new(),
            last_event_id: cid.hash().to_bytes(),
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
                    let signer = state.recovery_key_digest.to_key(&payload.signer_key)?;
                    signer.verify(&log.payload, &log.proof)?;
                    state.recovery_key_digest = Idp2pKeyDigest::from_bytes(change)?;
                }
                Change::Events(change) => {
                    let signer = state.next_key_digest.to_key(&payload.signer_key)?;
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
                    valid_at: timestamp,
                    expired_at: None,
                    key: Idp2pKey::from_bytes(key)?,
                };
                self.assertion_keys.push(assertion_method);
            }
            EventType::CreateAuthenticationKey(key) => {
                let previous_key = self.authentication_keys.last_mut();
                if let Some(previous_key) = previous_key {
                    previous_key.expired_at = Some(timestamp);
                }
                let authentication_method = KeyState {
                    valid_at: timestamp,
                    expired_at: None,
                    key: Idp2pKey::from_bytes(key)?,
                };
                self.authentication_keys.push(authentication_method);
            }
            EventType::CreateAgreementKey(key) => {
                let previous_key = self.agreement_keys.last_mut();
                if let Some(previous_key) = previous_key {
                    previous_key.expired_at = Some(timestamp);
                }
                let agreement_method = AgreementKeyState {
                    valid_at: timestamp,
                    expired_at: None,
                    key: Idp2pAgreementKey::from_bytes(key)?,
                };
                self.agreement_keys.push(agreement_method);
            }
            EventType::RevokeAssertionKey(kid) => {
                let key = self.assertion_keys.iter().find(|k| k == kid);
                if let Some(previous_key) = previous_key {
                    previous_key.expired_at = Some(timestamp);
                }
            }
            EventType::RevokeAuthenticationKey(kid) => {
                let key = self.authentication_keys.iter().find(|k| k == kid);
                if let Some(previous_key) = previous_key {
                    previous_key.expired_at = Some(timestamp);
                }
            }
            EventType::RevokeAgreementKey(kid) => {
                let key = self.agreement_keys.iter().find(|k| k == kid);
                if let Some(previous_key) = previous_key {
                    previous_key.expired_at = Some(timestamp);
                }
            }
            EventType::SetProof(proof) => {
                self.proofs.entry(proof.key).or_insert(proof.value);
            }
        }
        Ok(())
    }
}
