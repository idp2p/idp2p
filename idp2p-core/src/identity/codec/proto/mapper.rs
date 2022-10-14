use idp2p_common::multi::id::Idp2pId;
use prost::Message;

use crate::{
    error::Idp2pError,
    identity::{IdEvent, state::AssertionPublicKeyState},
    idp2p_proto::{
        self, identity_event::EventType, EventLog, EventLogPayload, IdentityEvent, Idp2pMultiKey,
        Idp2pProof,
    },
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
            IdEvent::CreateAssertionKey { id, multi_bytes } => IdentityEvent {
                event_type: Some(EventType::CreateAssertionKey(Idp2pMultiKey {
                    id: id,
                    bytes: multi_bytes,
                })),
            },
            IdEvent::CreateAuthenticationKey { id, multi_bytes } => IdentityEvent {
                event_type: Some(EventType::CreateAuthenticationKey(Idp2pMultiKey {
                    id: id,
                    bytes: multi_bytes,
                })),
            },
            IdEvent::CreateAgreementKey { id, multi_bytes } => IdentityEvent {
                event_type: Some(EventType::CreateAgreementKey(Idp2pMultiKey {
                    id: id,
                    bytes: multi_bytes,
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


impl IdentityStateEventMapper<EventType> for IdentityState {
    fn map_event(&mut self, timestamp: i64, event: EventType) -> Result<(), Idp2pError> {
        match event {
            EventType::CreateAssertionKey(key) => {
                let previous_key = self.assertion_keys.last_mut();
                if let Some(previous_key) = previous_key {
                    previous_key.expired_at = Some(timestamp);
                }
                let assertion_state = AssertionPublicKeyState {
                    id: key.id,
                    valid_at: timestamp,
                    expired_at: None,
                    key_bytes: key.bytes,
                };
                self.assertion_keys.push(assertion_state);
            }
            EventType::CreateAuthenticationKey(key) => {
                let previous_key = self.authentication_keys.last_mut();
                if let Some(previous_key) = previous_key {
                    previous_key.expired_at = Some(timestamp);
                }
                let authentication_state = AuthenticationPublicKeyState {
                    id: key.id,
                    valid_at: timestamp,
                    expired_at: None,
                    key_bytes: key.bytes,
                };
                self.authentication_keys.push(authentication_state);
            }
            EventType::CreateAgreementKey(key) => {
                let previous_key = self.agreement_keys.last_mut();
                if let Some(previous_key) = previous_key {
                    previous_key.expired_at = Some(timestamp);
                }
                let agreement_state = AgreementPublicKeyState {
                    id: key.id,
                    valid_at: timestamp,
                    expired_at: None,
                    key_bytes: key.bytes,
                };
                self.agreement_keys.push(agreement_state);
            }
            EventType::RevokeAssertionKey(kid) => {
                let key = self.assertion_keys.iter_mut().find(|k| k.id == kid);
                if let Some(key) = key {
                    key.expired_at = Some(timestamp);
                }
            }
            EventType::RevokeAuthenticationKey(kid) => {
                let key = self.authentication_keys.iter_mut().find(|k| k.id == kid);
                if let Some(key) = key {
                    key.expired_at = Some(timestamp);
                }
            }
            EventType::RevokeAgreementKey(kid) => {
                let key = self.agreement_keys.iter_mut().find(|k| k.id == kid);
                if let Some(key) = key {
                    key.expired_at = Some(timestamp);
                }
            }
            EventType::SetProof(proof) => {
                let entry = self.proofs.get_mut(&proof.key);
                if let Some(entry) = entry {
                    entry.expired_at = Some(timestamp);
                }
                self.proofs.insert(
                    proof.key,
                    ProofState {
                        valid_at: timestamp,
                        expired_at: None,
                        value: proof.value,
                    },
                );
            }
        }
        Ok(())
    }
}
