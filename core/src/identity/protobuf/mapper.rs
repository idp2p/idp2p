use crate::{
    identity::{input::IdEvents, doc::IdentityDocument},
    idp2p_proto::{self, identity_event::EventType, EventLog, EventLogPayload},
    IdentityError,
};
use idp2p_common::{
    anyhow::{bail, Result},
    digest::Idp2pDigest,
    digest::Idp2pKeyDigest,
    key::{Idp2pAgreementKey, Idp2pKey},
    ED25519_CODE, SHA256_CODE, X25519_CODE,
};
use prost::Message;

pub trait EventLogResolver {
    fn try_resolve_payload(&self, event_id: &Idp2pDigest) -> Result<EventLogPayload>;
}

impl EventLogResolver for EventLog {
    fn try_resolve_payload(&self, event_id: &Idp2pDigest) -> Result<EventLogPayload> {
        let digest = self
            .event_id
            .to_owned()
            .ok_or(IdentityError::InvalidProtobuf)?;
        let digest: Idp2pDigest = digest.try_into()?;
        if !digest.is_hash_of(&self.payload) {
            bail!(IdentityError::InvalidId)
        }
        let payload = EventLogPayload::decode(&*self.payload)?;
        let prev: Idp2pDigest = payload
            .clone()
            .previous
            .ok_or(IdentityError::InvalidProtobuf)?
            .try_into()?;
        if prev != *event_id {
            bail!(IdentityError::InvalidPrevious)
        }
        Ok(payload)
    }
}

impl Into<Vec<idp2p_proto::IdentityEvent>> for IdEvents {
    fn into(self) -> Vec<idp2p_proto::IdentityEvent> {
        let mut events: Vec<idp2p_proto::IdentityEvent> = vec![];
        if let Some(authentication_key) = self.authentication_key {
            events.push(idp2p_proto::IdentityEvent {
                event_type: Some(EventType::CreateAuthenticationKey(
                    authentication_key.into(),
                )),
            });
        }
        if let Some(agreement_key) = self.agreement_key {
            events.push(idp2p_proto::IdentityEvent {
                event_type: Some(EventType::CreateAgreementKey(agreement_key.into())),
            });
        }
        if let Some(assertion_key) = self.assertion_key {
            events.push(idp2p_proto::IdentityEvent {
                event_type: Some(EventType::CreateAssertionKey(assertion_key.into())),
            });
        }

        for (k, v) in self.proofs {
            events.push(idp2p_proto::IdentityEvent {
                event_type: Some(EventType::SetProof(idp2p_proto::Idp2pProof {
                    key: k,
                    value: v,
                })),
            });
        }
        events
    }
}

impl Into<IdentityDocument> for IdentityState {
    fn into(self) -> IdentityDocument {
        for assetion_key in self.assertion_keys {}
        let doc = IdentityDocument {
            context: vec![
                "https://www.w3.org/ns/did/v1".to_string(),
                "https://w3id.org/security/suites/ed25519-2020/v1".to_string(),
                "https://w3id.org/security/suites/x25519-2020/v1".to_string(),
            ],
            id: format!("did:p2p:{}", idp2p_common::encode(&self.id)),
            controller: format!("did:p2p:{}", idp2p_common::encode(&self.id)),
            verification_method: todo!(),
            assertion_method: todo!(),
            authentication: todo!(),
            key_agreement: todo!(),
        };
        todo!()
    }
}