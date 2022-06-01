use std::vec;

use cid::multihash::Multihash;
use prost::Message;

use crate::{
    identity::{error::IdentityError, IdEvents},
    idp2p_proto::{
        identity_event::EventType, EventLog, EventLogPayload, IdentityEvent, Idp2pProof,
        Idp2pVerificationKey,
    },
    multi::hash::Idp2pHash,
};

pub trait EventLogResolver {
    fn try_resolve_payload(&self, event_id: &[u8]) -> Result<EventLogPayload, IdentityError>;
}

impl EventLogResolver for EventLog {
    fn try_resolve_payload(&self, last_event_id: &[u8]) -> Result<EventLogPayload, IdentityError> {
        let mh = Multihash::from_bytes(&self.event_id)?;
        let hash = Idp2pHash::try_from(mh.code())?;
        hash.ensure(mh, &self.payload)?;
        let payload = EventLogPayload::decode(&*self.payload)?;
        if payload.previous != *last_event_id {
            return Err(IdentityError::InvalidPrevious);
        }
        Ok(payload)
    }
}

impl Into<Vec<IdentityEvent>> for IdEvents {
    fn into(self) -> Vec<IdentityEvent> {
        let mut events: Vec<IdentityEvent> = vec![];
        if let Some(assertion_key) = self.assertion_key {
            events.push(IdentityEvent {
                event_type: Some(EventType::CreateAssertionKey(Idp2pVerificationKey {
                    id: assertion_key.to_id(),
                    value: assertion_key.to_bytes(),
                })),
            });
        }
        if let Some(authentication_key) = self.authentication_key {
            events.push(IdentityEvent {
                event_type: Some(EventType::CreateAuthenticationKey(Idp2pVerificationKey {
                    id: authentication_key.to_id(),
                    value: authentication_key.to_bytes(),
                })),
            });
        }
        if let Some(agreement_key) = self.agreement_key {
            events.push(IdentityEvent {
                event_type: Some(EventType::CreateAgreementKey(Idp2pVerificationKey {
                    id: agreement_key.to_id(),
                    value: agreement_key.to_bytes(),
                })),
            });
        }
        for (k, v) in self.proofs {
            events.push(IdentityEvent {
                event_type: Some(EventType::SetProof(Idp2pProof { key: k, value: v })),
            });
        }
        events
    }
}
