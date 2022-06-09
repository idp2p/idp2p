use idp2p_common::{ multi::hash::Idp2pHash};
use prost::Message;

use crate::{
    identity::{error::IdentityError, models::IdEvent},
    idp2p_proto::{
        identity_event::EventType, EventLog, EventLogPayload, IdentityEvent, Idp2pProof,
        Idp2pVerificationKey,
    },
};

/// Resolve event payload from encoded protobuf
pub trait EventLogResolver {
    fn try_resolve_payload(&self, event_id: &[u8]) -> Result<EventLogPayload, IdentityError>;
}

impl EventLogResolver for EventLog {
    fn try_resolve_payload(&self, last_event_id: &[u8]) -> Result<EventLogPayload, IdentityError> {
        // Get multihash of last_event_id
        let mh = Idp2pHash::from_bytes(&self.event_id)?;
        let hash = Idp2pHash::try_from(mh.code())?;
        // Ensure generated id equals event_id
        hash.ensure(mh, &self.payload)?;
        let payload = EventLogPayload::decode(&*self.payload)?;
        // Previous event_id should match with last_event_id of state.
        // Because all identity events point previous event.
        // First event points to inception event
        if payload.previous != *last_event_id {
            return Err(IdentityError::InvalidPrevious);
        }
        Ok(payload)
    }
}

impl Into<IdentityEvent> for IdEvent {
    fn into(self) -> IdentityEvent {
        match self {
            IdEvent::CreateAssertionKey { id, key} => IdentityEvent {
                event_type: Some(EventType::CreateAssertionKey(Idp2pVerificationKey {
                    id: id,
                    value: key.to_bytes(),
                })),
            },
            IdEvent::CreateAuthenticationKey{ id, key} => IdentityEvent {
                event_type: Some(EventType::CreateAuthenticationKey(Idp2pVerificationKey {
                    id: id,
                    value: key.to_bytes(),
                })),
            },
            IdEvent::CreateAgreementKey{ id, key} => IdentityEvent {
                event_type: Some(EventType::CreateAgreementKey(Idp2pVerificationKey {
                    id: id,
                    value: key.to_bytes(),
                })),
            },
            IdEvent::SetProof { key, value } => IdentityEvent {
                event_type: Some(EventType::SetProof(Idp2pProof { key, value })),
            },
            IdEvent::RevokeAssertionKey(kid) => IdentityEvent {
                event_type: Some(EventType::RevokeAssertionKey(kid)),
            },
            IdEvent::RevokeAuthenticationKey(kid) => IdentityEvent {
                event_type: Some(EventType::RevokeAuthenticationKey(kid)),
            },
            IdEvent::RevokeAgreementKey(kid) => IdentityEvent {
                event_type: Some(EventType::RevokeAgreementKey(kid)),
            },
        }
    }
}
