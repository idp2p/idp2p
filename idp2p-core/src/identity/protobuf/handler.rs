use idp2p_common::{
    anyhow::Result, chrono::Utc, cid::Cid, digest::Idp2pDigest, key::Idp2pKey, multi_id::Idp2pCid,
    Idp2pCodec,
};
use prost::Message;
use std::collections::HashMap;

use crate::{
    identity::{
        input::{ChangeInput, CreateIdentityInput, RecoverInput},
        state::{IdentityState, IdentityStateEventHandler},
        IdentityBehaviour,
    },
    idp2p_proto::{
        event_log_payload::{Change, IdentityEvents},
        EventLog, EventLogPayload, Identity, IdentityInception, Microledger,
    },
    IdentityError,
};

use super::mapper::EventLogResolver;

impl IdentityBehaviour for Identity {
    fn create(input: CreateIdentityInput) -> Result<Self> {
        let mut inception = IdentityInception {
            version: 1,
            timestamp: Utc::now().timestamp(),
            next_key_digest: Some(input.next_key_digest.into()),
            recovery_key_digest: Some(input.recovery_key_digest.into()),
            events: vec![],
        };
        inception.events = input.events.into();
        let inception_bytes = inception.encode_to_vec();
        let cid = Cid::new_cid(&inception_bytes, Idp2pCodec::Protobuf);
        let microledger = Microledger {
            inception: inception_bytes,
            event_logs: vec![],
        };
        let did = Identity {
            id: cid.into(),
            microledger: Some(microledger),
        };
        Ok(did)
    }

    fn change(&mut self, input: ChangeInput) -> Result<()> {
        let state = self.verify()?;
        let microledger = self
            .microledger
            .as_mut()
            .ok_or(IdentityError::InvalidProtobuf)?;
        let signer_key: Idp2pKey = input.signer.clone().into();
        let payload = EventLogPayload {
            version: 1,
            previous: Some(state.event_id.into()),
            signer_key: signer_key.to_bytes(),
            next_key_digest: Some(input.next_key_digest.into()),
            timestamp: Utc::now().timestamp(),
            change: Some(Change::Events(IdentityEvents {
                events: input.events.into(),
            })),
        };
        let payload_bytes = payload.encode_to_vec();
        let proof = input.signer.sign(&payload_bytes);
        let event_log = EventLog {
            event_id: Some(Idp2pDigest::new(&payload_bytes).into()),
            payload: payload_bytes,
            proof: proof,
        };
        microledger.event_logs.push(event_log);
        Ok(())
    }

    fn recover(&mut self, input: RecoverInput) -> Result<()> {
        todo!()
    }

    fn verify(&self) -> Result<IdentityState> {
        let microledger = self
            .microledger
            .as_ref()
            .ok_or(IdentityError::InvalidProtobuf)?;
        let cid: Cid = self.id.to_vec().try_into()?;
        cid.ensure(&microledger.inception, Idp2pCodec::Protobuf)?;
        // Get inception of microledger
        let inception = IdentityInception::decode(&*microledger.inception)?;

        let next_key_digest = inception
            .next_key_digest
            .ok_or(IdentityError::InvalidProtobuf)?;
        let recovery_key_digest = inception
            .recovery_key_digest
            .ok_or(IdentityError::InvalidProtobuf)?;
        // Init current state to handle events
        let mut state = IdentityState {
            id: self.id.clone(),
            event_id: Idp2pDigest::new(&microledger.inception),
            next_key_digest: next_key_digest.try_into()?,
            recovery_key_digest: recovery_key_digest.try_into()?,
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
            let payload = log.try_resolve_payload(&state.event_id)?;
            let change = payload.change.ok_or(IdentityError::InvalidProtobuf)?;
            match change {
                Change::Recover(change) => {
                    let signer =
                        Idp2pKey::new(state.recovery_key_digest.code(), &payload.signer_key)?;
                    signer.verify(&log.payload, &log.proof)?;
                    state.recovery_key_digest = change.try_into()?;
                }
                Change::Events(change) => {
                    let signer = Idp2pKey::new(state.next_key_digest.code(), &payload.signer_key)?;
                    signer.verify(&log.payload, &log.proof)?;
                    for event in change.events {
                        let event = event.event_type.ok_or(IdentityError::InvalidProtobuf)?;
                        state.handle_event(payload.timestamp, event)?;
                    }
                }
            }
            state.next_key_digest = payload
                .next_key_digest
                .ok_or(IdentityError::InvalidProtobuf)?
                .try_into()?;
            state.event_id = log
                .event_id
                .clone()
                .ok_or(IdentityError::InvalidProtobuf)?
                .try_into()?;
        }
        Ok(state)
    }
}
