use crate::{
    identity::{
        error::IdentityError,
        state::{
            AgreementKeyState, IdentityState, IdentityStateEventHandler, KeyState, ProofState,
        },
    },
    idp2p_proto::identity_event::EventType,
};

impl IdentityStateEventHandler<EventType> for IdentityState {
    fn handle_event(&mut self, timestamp: i64, event: EventType) -> Result<(), IdentityError> {
        match event {
            EventType::CreateAssertionKey(key) => {
                let previous_key = self.assertion_keys.last_mut();
                if let Some(previous_key) = previous_key {
                    previous_key.expired_at = Some(timestamp);
                }
                let assertion_method = KeyState {
                    id: key.id,
                    valid_at: timestamp,
                    expired_at: None,
                    key: key.value,
                };
                self.assertion_keys.push(assertion_method);
            }
            EventType::CreateAuthenticationKey(key) => {
                let previous_key = self.authentication_keys.last_mut();
                if let Some(previous_key) = previous_key {
                    previous_key.expired_at = Some(timestamp);
                }
                let authentication_method = KeyState {
                    id: key.id,
                    valid_at: timestamp,
                    expired_at: None,
                    key: key.value,
                };
                self.authentication_keys.push(authentication_method);
            }
            EventType::CreateAgreementKey(key) => {
                let previous_key = self.agreement_keys.last_mut();
                if let Some(previous_key) = previous_key {
                    previous_key.expired_at = Some(timestamp);
                }
                let agreement_method = AgreementKeyState {
                    id: key.id,
                    valid_at: timestamp,
                    expired_at: None,
                    key: key.value,
                };
                self.agreement_keys.push(agreement_method);
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
