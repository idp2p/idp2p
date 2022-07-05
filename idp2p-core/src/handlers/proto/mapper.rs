use idp2p_common::multi::hasher::Idp2pHasher;
use prost::Message;

use crate::{idp2p_proto::{
    identity_event::EventType, EventLog, EventLogPayload, IdentityEvent, Idp2pProof,
    Idp2pVerificationKey,
}, error::Idp2pError, identity::IdEvent};

/// Resolve event payload from encoded protobuf
pub trait EventLogResolver {
    fn try_resolve_payload(&self, event_id: &[u8]) -> Result<EventLogPayload, Idp2pError>;
}

impl EventLogResolver for EventLog {
    fn try_resolve_payload(&self, last_event_id: &[u8]) -> Result<EventLogPayload, Idp2pError> {
        // Get multihash of last_event_id
        let mh = Idp2pHasher::from_bytes(&self.event_id)?;
        let hasher = Idp2pHasher::try_from(mh.code())?;
        // Ensure generated id equals event_id
        hasher.ensure(mh, &self.payload)?;
        let payload = EventLogPayload::decode(self.payload.as_slice())?;
        // Previous event_id should match with last_event_id of state.
        // Because all identity events point previous event.
        // First event points to inception event
        if payload.previous != *last_event_id {
            return Err(Idp2pError::InvalidPrevious);
        }
        Ok(payload)
    }
}

impl Into<IdentityEvent> for IdEvent {
    fn into(self) -> IdentityEvent {
        match self {
            IdEvent::CreateAssertionKey { id, key } => IdentityEvent {
                event_type: Some(EventType::CreateAssertionKey(Idp2pVerificationKey {
                    id: id,
                    value: key,
                })),
            },
            IdEvent::CreateAuthenticationKey { id, key } => IdentityEvent {
                event_type: Some(EventType::CreateAuthenticationKey(Idp2pVerificationKey {
                    id: id,
                    value: key,
                })),
            },
            IdEvent::CreateAgreementKey { id, key } => IdentityEvent {
                event_type: Some(EventType::CreateAgreementKey(Idp2pVerificationKey {
                    id: id,
                    value: key,
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
