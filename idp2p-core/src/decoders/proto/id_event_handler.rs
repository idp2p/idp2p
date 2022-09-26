use crate::{
    error::Idp2pError,
    id_state::{
        AgreementPublicKeyState, AssertionPublicKeyState, AuthenticationPublicKeyState,
        IdentityState, IdentityStateEventHandler, ProofState,
    },
    idp2p_proto::identity_event::EventType,
};

impl IdentityStateEventHandler<EventType> for IdentityState {
    fn handle_event(&mut self, timestamp: i64, event: EventType) -> Result<(), Idp2pError> {
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
