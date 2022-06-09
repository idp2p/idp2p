use crate::{
    identity::{
        error::IdentityError,
        models::IdEvent,
        state::{
            AgreementKeyState, IdentityState, IdentityStateEventHandler, KeyState, ProofState,
        },
    }
};

impl IdentityStateEventHandler<IdEvent> for IdentityState {
    fn handle_event(&mut self, timestamp: i64, event: IdEvent) -> Result<(), IdentityError> {
        match event {
            IdEvent::CreateAssertionKey { id, key } => {
                let previous_key = self.assertion_keys.last_mut();
                if let Some(previous_key) = previous_key {
                    previous_key.expired_at = Some(timestamp);
                }
                let assertion_method = KeyState {
                    id: id,
                    valid_at: timestamp,
                    expired_at: None,
                    key: key,
                };
                self.assertion_keys.push(assertion_method);
            }
            IdEvent::CreateAuthenticationKey { id, key } => {
                let previous_key = self.authentication_keys.last_mut();
                if let Some(previous_key) = previous_key {
                    previous_key.expired_at = Some(timestamp);
                }
                let authentication_method = KeyState {
                    id: id,
                    valid_at: timestamp,
                    expired_at: None,
                    key: key,
                };
                self.authentication_keys.push(authentication_method);
            }
            IdEvent::CreateAgreementKey { id, key } => {
                let previous_key = self.agreement_keys.last_mut();
                if let Some(previous_key) = previous_key {
                    previous_key.expired_at = Some(timestamp);
                }
                let agreement_method = AgreementKeyState {
                    id: id,
                    valid_at: timestamp,
                    expired_at: None,
                    key: key,
                };
                self.agreement_keys.push(agreement_method);
            }
            IdEvent::RevokeAssertionKey(kid) => {
                let key = self.assertion_keys.iter_mut().find(|k| k.id == kid);
                if let Some(key) = key {
                    key.expired_at = Some(timestamp);
                }
            }
            IdEvent::RevokeAuthenticationKey(kid) => {
                let key = self.authentication_keys.iter_mut().find(|k| k.id == kid);
                if let Some(key) = key {
                    key.expired_at = Some(timestamp);
                }
            }
            IdEvent::RevokeAgreementKey(kid) => {
                let key = self.agreement_keys.iter_mut().find(|k| k.id == kid);
                if let Some(key) = key {
                    key.expired_at = Some(timestamp);
                }
            }
            IdEvent::SetProof { key, value } => {
                let entry = self.proofs.get_mut(&key);
                if let Some(entry) = entry {
                    entry.expired_at = Some(timestamp);
                }
                self.proofs.insert(
                    key,
                    ProofState {
                        valid_at: timestamp,
                        expired_at: None,
                        value: value,
                    },
                );
            }
        }
        Ok(())
    }
}
