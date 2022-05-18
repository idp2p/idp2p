use std::collections::HashMap;

use idp2p_common::{
    anyhow::Result,
    chrono::Utc,
    cid::{
        multihash::{Code, MultihashDigest},
        Cid,
    },
    Idp2pCodec, multi_id::Idp2pCid,
};

use crate::{
    identity::{
        input::{ChangeInput, CreateIdentityInput, RecoverInput},
        state::IdentityState,
        IdentityBehaviour,
    },
    IdentityError,
};

use super::{EventLog, EventLogChange, EventLogPayload, Identity, IdentityInception, Microledger};

impl IdentityBehaviour for Identity {
    fn create(input: CreateIdentityInput) -> Result<Self> {
        let inception = IdentityInception {
            version: 1,
            timestamp: Utc::now().timestamp(),
            next_key_digest: input.next_key_digest,
            recovery_key_digest: input.recovery_key_digest,
            events: input.events.into(),
        };
        let inception_bytes = idp2p_common::serde_json::to_vec(&inception)?;
        let cid = Cid::new_cid(&inception_bytes, Idp2pCodec::Json);
        let microledger = Microledger {
            inception: inception,
            event_logs: vec![],
        };
        let did = Identity {
            id: cid.to_string(),
            microledger: microledger,
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
            previous: state.event_id,
            signer_key: signer_key.into(),
            next_key_digest: Some(input.next_key_digest.into()),
            timestamp: Utc::now().timestamp(),
            change: Change::Events(IdentityEvents {
                events: input.events.into(),
            }),
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
        let inception_bytes = idp2p_common::serde_json::to_vec(&self.microledger.inception)?;
        let cid = Cid::new_cid(&inception_bytes, Idp2pCodec::Json);
        cid.ensure()
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
                EventLogChange::Recover { digest } => {
                    let signer = state.recovery_key_digest.to_key(&payload.signer_key)?;
                    signer.verify(&log.payload, &log.proof)?;
                    state.recovery_key_digest = change.try_into()?;
                }
                EventLogChange::Events { events } => {
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
