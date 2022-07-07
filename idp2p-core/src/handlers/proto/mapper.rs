use idp2p_common::multi::id::Idp2pId;
use prost::Message;

use crate::{
    error::Idp2pError,
    identity::IdEvent,
    idp2p_proto::{
        identity_event::EventType, EventLog, EventLogPayload, IdentityEvent, Idp2pProof,
        Idp2pVerificationKey, self,
    }, id_message::IdMessage,
};

/// Resolve event payload from encoded protobuf
pub trait EventLogResolver {
    fn try_resolve_payload(&self) -> Result<EventLogPayload, Idp2pError>;
}

impl EventLogResolver for EventLog {
    fn try_resolve_payload(&self) -> Result<EventLogPayload, Idp2pError> {
        let id = Idp2pId::from_bytes(&self.id)?;
        id.ensure(&self.payload)?;
        let payload = EventLogPayload::decode(self.payload.as_slice())?;
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

impl Into<IdMessage> for idp2p_proto::IdMessage {
    fn into(self) -> IdMessage {
        IdMessage{
            from: self.from,
            to: self.to,
            signer_kid: self.signer_kid,
            proof: self.proof,
            created_at: self.created_at,
            body: self.body,
        }
    }
}