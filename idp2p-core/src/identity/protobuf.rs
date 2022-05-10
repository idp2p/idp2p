use std::collections::HashMap;

use crate::IdentityError;

use super::{
    core::{
        CreateIdentityInput, IdentityBehaviour, IdentityEvent, IdentityState, IdentityVerifyResult,
    },
    did_doc::IdentityDocument,
    idp2p_proto::{
        self, event_log_payload::EventLogChange, identity_event::IdentityEventType,
        EventLogPayload, Identity, IdentityInception,
    },
};
use idp2p_common::{
    anyhow::{bail, Result},
    cid::Cid,
    hash::{Idp2pCodec, Idp2pHash},
    key::Idp2pKey,
    key_digest::Idp2pKeyDigest,
    secret::Idp2pSecret,
};
use prost::Message;

impl TryInto<idp2p_proto::Idp2pKeyDigest> for Idp2pKeyDigest {
    type Error = idp2p_common::anyhow::Error;

    fn try_into(self) -> Result<idp2p_proto::Idp2pKeyDigest, Self::Error> {
        todo!()
    }
}

impl TryInto<Idp2pKeyDigest> for idp2p_proto::Idp2pKeyDigest {
    type Error = idp2p_common::anyhow::Error;

    fn try_into(self) -> Result<Idp2pKeyDigest, Self::Error> {
        todo!()
    }
}

impl IdentityBehaviour for Identity {
    fn add_events(&mut self, signer: Idp2pSecret, events: Vec<IdentityEvent>) -> Result<()> {
        todo!()
    }

    fn create(input: CreateIdentityInput) -> Result<Self> {
        todo!()
    }

    fn recover(&mut self, signer: Idp2pSecret, rec_digest: Idp2pKeyDigest) -> Result<()> {
        todo!()
    }

    fn to_document(&self) -> Result<IdentityDocument> {
        todo!()
    }

    fn verify(&self) -> Result<IdentityVerifyResult> {
        let microledger = self
            .microledger
            .as_ref()
            .ok_or(IdentityError::InvalidProtobuf)?;
        // Get inception of microledger
        let inception = IdentityInception::decode(&*microledger.inception)?;
        let cid = Cid::try_from(self.id.clone())?;
        let hash_alg = Idp2pHash::try_from(cid.hash().code())?;
        let expected_cid = hash_alg.generate_cid(&microledger.inception, cid.codec());
        if self.id != expected_cid {
            bail!(IdentityError::InvalidId)
        }

        let next_key_digest = inception
            .next_key_digest
            .ok_or(IdentityError::InvalidProtobuf)?;
        let recovery_key_digest = inception
            .recovery_key_digest
            .ok_or(IdentityError::InvalidProtobuf)?;
        // Init current state to handle events
        let mut state = IdentityState {
            event_id: hash_alg.generate_id(&microledger.inception),
            next_key_digest: next_key_digest.try_into()?,
            recovery_key_digest: recovery_key_digest.try_into()?,
        };
        // Create initial verify result
        let mut result = IdentityVerifyResult {
            id: self.id.clone(),
            next_key_digest: state.next_key_digest.clone(),
            recovery_key_digest: state.recovery_key_digest.clone(),
            assertion_keys: vec![],
            authentication_key: None,
            agreement_key: None,
            proofs: HashMap::new(),
        };
        // Handle initial events
        for event in inception.events {
            let event = event
                .identity_event_type
                .ok_or(IdentityError::InvalidProtobuf)?;
            handle_id_event(event, &mut state, &mut result);
        }
        for log in &microledger.event_logs {
            let payload = EventLogPayload::decode(&*log.payload)?;
            let change = payload
                .event_log_change
                .ok_or(IdentityError::InvalidProtobuf)?;
            match change {
                EventLogChange::Recover(change) => {
                    let signer = state.recovery_key_digest.to_key(&payload.signer_key)?;
                    signer.verify(&log.payload, &log.proof)?;
                    state.recovery_key_digest = change.try_into()?;
                }
                EventLogChange::Events(change) => {
                    let signer = state.next_key_digest.to_key(&payload.signer_key)?;
                    signer.verify(&log.payload, &log.proof)?;
                    for event in change.events {
                        let event = event
                            .identity_event_type
                            .ok_or(IdentityError::InvalidProtobuf)?;
                        handle_id_event(event, &mut state, &mut result);
                    }
                }
            }
            state.next_key_digest = payload
                .next_key_digest
                .ok_or(IdentityError::InvalidProtobuf)?
                .try_into()?;
            state.event_id = log.event_id.clone();
        }
        todo!()
    }
}

fn handle_id_event(
    event: IdentityEventType,
    state: &mut IdentityState,
    result: &mut IdentityVerifyResult,
) {
    match event {
        IdentityEventType::CreateAssertionKey(bytes) => {}
        IdentityEventType::CreateAuthenticationKey(bytes) => {}
        IdentityEventType::CreateAgreementKey(bytes) => {}
        IdentityEventType::RevokeAssertionKey(bytes) => {}
        IdentityEventType::RevokeAuthenticationKey(bytes) => {}
        IdentityEventType::RevokeAgreementKey(bytes) => {}
        IdentityEventType::SetProof(proof) => {}
    }
}
