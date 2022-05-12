use std::collections::HashMap;

use crate::IdentityError;

use super::{
    core::{
        AssertionKeyState, AuthenticationKeyState, CreateIdentityInput, IdentityBehaviour,
        IdentityEvents, IdentityState,
    }
};
use crate::idp2p_proto::{
    self, event_log_payload::Change, identity_event::EventType, EventLogPayload, Identity,
    IdentityInception,
};
use idp2p_common::{
    agreement_key::Idp2pAgreementKey,
    anyhow::{bail, Result},
    chrono::Utc,
    cid::{
        multihash::{Code, Multihash, MultihashDigest},
        Cid,
    },
    key::Idp2pKey,
    key_digest::Idp2pKeyDigest,
    secret::Idp2pSecret,
    Idp2pCodec, Idp2pHasher,
};
use prost::Message;

impl Into<idp2p_proto::Idp2pKeyDigest> for Idp2pKeyDigest {
    fn into(self) -> idp2p_proto::Idp2pKeyDigest {
        match self {
            Idp2pKeyDigest::Idp2pEd25519 { mh } => idp2p_proto::Idp2pKeyDigest {
                key_type: 0xed,
                digest: mh.to_bytes(),
            },
        }
    }
}

impl TryInto<Idp2pKeyDigest> for idp2p_proto::Idp2pKeyDigest {
    type Error = idp2p_common::anyhow::Error;

    fn try_into(self) -> Result<Idp2pKeyDigest, Self::Error> {
        Idp2pKeyDigest::try_from(self.key_type as u64, &self.digest)
    }
}

impl Into<idp2p_proto::Idp2pKey> for Idp2pKey {
    fn into(self) -> idp2p_proto::Idp2pKey {
        match self {
            Idp2pKey::Idp2pEd25519 { public } => idp2p_proto::Idp2pKey {
                key_type: 0xed,
                public: public.to_bytes().to_vec(),
            },
        }
    }
}

impl TryInto<Idp2pKey> for idp2p_proto::Idp2pKey {
    type Error = idp2p_common::anyhow::Error;

    fn try_into(self) -> Result<Idp2pKey, Self::Error> {
        Idp2pKey::try_from(self.key_type as u64, &self.public)
    }
}

impl Into<idp2p_proto::Idp2pAgreementKey> for Idp2pAgreementKey {
    fn into(self) -> idp2p_proto::Idp2pAgreementKey {
        match self {
            Idp2pAgreementKey::Idp2pX25519 { public } => idp2p_proto::Idp2pAgreementKey {
                key_type: 0xed,
                public: public.to_bytes().to_vec(),
            },
        }
    }
}

impl TryInto<Idp2pAgreementKey> for idp2p_proto::Idp2pAgreementKey {
    type Error = idp2p_common::anyhow::Error;

    fn try_into(self) -> Result<Idp2pAgreementKey, Self::Error> {
        Idp2pAgreementKey::try_from(self.key_type as u64, &self.public)
    }
}

impl Into<Vec<idp2p_proto::IdentityEvent>> for IdentityEvents {
    fn into(self) -> Vec<idp2p_proto::IdentityEvent> {
        let mut events: Vec<idp2p_proto::IdentityEvent> = vec![];
        events.push(idp2p_proto::IdentityEvent {
            event_type: Some(
                idp2p_proto::identity_event::EventType::CreateAuthenticationKey(
                    self.authentication_key.into(),
                ),
            ),
        });
        events.push(idp2p_proto::IdentityEvent {
            event_type: Some(idp2p_proto::identity_event::EventType::CreateAgreementKey(
                self.agreement_key.into(),
            )),
        });
        for assertion_key in self.assertion_keys {
            events.push(idp2p_proto::IdentityEvent {
                event_type: Some(idp2p_proto::identity_event::EventType::CreateAssertionKey(
                    assertion_key.into(),
                )),
            });
        }
        for (k, v) in self.proofs {
            events.push(idp2p_proto::IdentityEvent {
                event_type: Some(idp2p_proto::identity_event::EventType::SetProof(
                    idp2p_proto::Idp2pProof { key: k, value: v },
                )),
            });
        }
        events
    }
}

trait EventLogResolver {
    fn try_resolve(&self, event_id: &[u8]) -> Result<EventLogPayload>;
}

impl EventLogResolver for idp2p_proto::EventLog {
    fn try_resolve(&self, event_id: &[u8]) -> Result<EventLogPayload> {
        let multi_id = Multihash::from_bytes(&self.event_id)?;
        if !multi_id.is_hash_of(&self.payload)? {
            bail!(IdentityError::InvalidId)
        }
        let payload = EventLogPayload::decode(&*self.payload)?;
        if payload.previous != event_id {
            bail!(IdentityError::InvalidPrevious)
        }
        Ok(payload)
    }
}

impl IdentityBehaviour for Identity {
    fn create(input: CreateIdentityInput) -> Result<Self> {
        let mut inception = idp2p_proto::IdentityInception {
            timestamp: Utc::now().timestamp(),
            next_key_digest: Some(input.next_key_digest.into()),
            recovery_key_digest: Some(input.recovery_key_digest.into()),
            events: vec![],
        };
        inception.events = input.events.into();
        let inception_bytes = inception.encode_to_vec();
        let mh = Code::Sha2_256.digest(&inception_bytes);
        let microledger = idp2p_proto::Microledger {
            inception: inception_bytes,
            event_logs: vec![],
        };
        let id = Cid::new_v1(Idp2pCodec::Protobuf as u64, mh);
        let did = idp2p_proto::Identity {
            id: id.to_bytes(),
            microledger: Some(microledger),
        };
        Ok(did)
    }

    fn add_events(&mut self, signer: Idp2pSecret, next_key_digest: Idp2pKeyDigest, input: IdentityEvents) -> Result<()> {
        let state = self.verify()?;
        let microledger = self
            .microledger
            .as_mut()
            .ok_or(IdentityError::InvalidProtobuf)?;
        let signer_key: Idp2pKey = signer.clone().into();
        let payload = idp2p_proto::EventLogPayload {
            previous: state.event_id,
            signer_key: signer_key.into(),
            next_key_digest: Some(next_key_digest.into()),
            timestamp: Utc::now().timestamp(),
            change: Some(Change::Events(
                idp2p_proto::event_log_payload::IdentityEvents {
                    events: input.into(),
                },
            )),
        };
        let payload_bytes = payload.encode_to_vec();
        let proof = signer.sign(&payload_bytes);
        let event_log = idp2p_proto::EventLog {
            event_id: Code::Sha2_256.digest(&payload_bytes).to_bytes(),
            payload: payload_bytes,
            proof: proof,
        };
        microledger.event_logs.push(event_log);
        Ok(())
    }

    fn recover(&mut self, signer: Idp2pSecret, next_key_digest: Idp2pKeyDigest, rec_digest: Idp2pKeyDigest) -> Result<()> {
        todo!()
    }

    fn verify(&self) -> Result<IdentityState> {
        let microledger = self
            .microledger
            .as_ref()
            .ok_or(IdentityError::InvalidProtobuf)?;
        ensure_cid(&self.id, &microledger.inception)?;
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
            handle_id_event(event, inception.timestamp, &mut state)?;
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
                        handle_id_event(event, payload.timestamp, &mut state)?;
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

fn handle_id_event(event: EventType, ts: i64, state: &mut IdentityState) -> Result<()> {
    match event {
        EventType::CreateAssertionKey(key) => {
            let previous_key = state.assertion_keys.last_mut();
            if let Some(previous_key) = previous_key {
                previous_key.expired_at = Some(ts);
            }
            let assertion_method = AssertionKeyState {
                valid_at: ts,
                expired_at: None,
                key: key.try_into()?,
            };
            state.assertion_keys.push(assertion_method);
        }
        EventType::CreateAuthenticationKey(key) => {
            let previous_key = state.authentication_keys.last_mut();
            if let Some(previous_key) = previous_key {
                previous_key.expired_at = Some(ts);
            }
            let authentication_method = AuthenticationKeyState {
                valid_at: ts,
                expired_at: None,
                key: key.try_into()?,
            };
            state.authentication_keys.push(authentication_method);
        }
        EventType::CreateAgreementKey(key) => {}
        EventType::RevokeAssertionKey(kid) => {}
        EventType::RevokeAuthenticationKey(kid) => {}
        EventType::RevokeAgreementKey(kid) => {}
        EventType::SetProof(proof) => {}
    }
    Ok(())
}

fn ensure_cid(cid: &[u8], inception: &[u8]) -> Result<()> {
    let cid: Cid = cid.to_vec().try_into()?;
    if cid.codec() != Idp2pCodec::Protobuf as u64 {
        bail!(IdentityError::InvalidId)
    }
    if !cid.hash().is_hash_of(inception)? {
        bail!(IdentityError::InvalidId)
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use idp2p_common::secret::EdSecret;

    use super::*;
    #[test]
    fn create_test() -> Result<()> {
        let secret = Idp2pSecret::Idp2p25519 {
            secret: EdSecret::new(),
        };
        let key: Idp2pKey = secret.clone().into();
        let key_digest: Idp2pKeyDigest = key.clone().into();
        let events = IdentityEvents {
            proofs: HashMap::new(),
            authentication_key: key.clone(),
            agreement_key: secret.clone().into(),
            assertion_keys: vec![],
        };
        let input = CreateIdentityInput {
            next_key_digest: key_digest.clone().into(),
            recovery_key_digest: key_digest.into(),
            events: events.clone(),
        };
        let mut did = Identity::create(input)?;
        eprintln!("{:?}", &did.id);
        did.add_events(secret.clone(), Into::<Idp2pKey>::into(secret).into(), events)?;
        let state = did.verify()?;
        //let inception = idp2p_proto::IdentityInception::decode(&*did.microledger.unwrap().inception);
        eprintln!("{:?}", state);
        Ok(())
    }
}
