use idp2p_common::{
    anyhow::Result,
    chrono::Utc,
    cid::{
        multihash::{Code, MultihashDigest},
        Cid,
    },
    key::Idp2pKey as IdKey,
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

use super::{mapper::EventLogResolver};

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
        let mh = Code::Sha2_256.digest(&inception_bytes);
        let id = Cid::new_v1(Idp2pCodec::Protobuf as u64, mh);
        let microledger = Microledger {
            inception: inception_bytes,
            event_logs: vec![],
        };
        let did = Identity {
            id: id.to_bytes(),
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
        let signer_key: IdKey = input.signer.clone().into();
        let payload = EventLogPayload {
            version: 1,
            previous: state.event_id,
            signer_key: signer_key.raw_bytes(),
            next_key_digest: Some(input.next_key_digest.into()),
            timestamp: Utc::now().timestamp(),
            change: Some(Change::Events(IdentityEvents {
                events: input.events.into(),
            })),
        };
        let payload_bytes = payload.encode_to_vec();
        let proof = input.signer.sign(&payload_bytes);
        let event_log = EventLog {
            event_id: Code::Sha2_256.digest(&payload_bytes).to_bytes(),
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
        // ensure_cid(&self.id, &microledger.inception)?;
        // Get inception of microledger
        let inception = IdentityInception::decode(&*microledger.inception)?;

        let next_key_digest = inception
            .next_key_digest
            .ok_or(IdentityError::InvalidProtobuf)?;
        let recovery_key_digest = inception
            .recovery_key_digest
            .ok_or(IdentityError::InvalidProtobuf)?;
        // Init current state to handle events
        let cid: Cid = self.id.to_vec().try_into()?;
        let mut state = IdentityState {
            id: self.id.clone(),
            event_id: cid.hash().to_bytes(),
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
            let payload = log.try_resolve(&state.event_id)?;
            let change = payload.change.ok_or(IdentityError::InvalidProtobuf)?;
            match change {
                Change::Recover(change) => {
                    let signer = state.recovery_key_digest.to_key(&payload.signer_key)?;
                    signer.verify(&log.payload, &log.proof)?;
                    state.recovery_key_digest = change.try_into()?;
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
            state.next_key_digest = payload
                .next_key_digest
                .ok_or(IdentityError::InvalidProtobuf)?
                .try_into()?;
            state.event_id = log.event_id.clone();
        }
        Ok(state)
    }
}
