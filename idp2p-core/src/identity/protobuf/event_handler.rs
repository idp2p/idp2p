use idp2p_common::anyhow::Result;
use crate::{
    identity::state::{
        KeyState, IdentityState, IdentityStateEventHandler,
    },
    idp2p_proto::identity_event::EventType,
};

impl IdentityStateEventHandler<EventType> for IdentityState {
    fn handle_event(&mut self, timestamp: i64, event: EventType) -> Result<()> {
        match event {
            EventType::CreateAssertionKey(key) => {
                let previous_key = self.assertion_keys.last_mut();
                if let Some(previous_key) = previous_key {
                    previous_key.expired_at = Some(timestamp);
                }
                let assertion_method = KeyState {
                    valid_at: timestamp,
                    expired_at: None,
                    key: key.try_into()?,
                };
                self.assertion_keys.push(assertion_method);
            }
            EventType::CreateAuthenticationKey(key) => {
                let previous_key = self.authentication_keys.last_mut();
                if let Some(previous_key) = previous_key {
                    previous_key.expired_at = Some(timestamp);
                }
                let authentication_method = KeyState {
                    valid_at: timestamp,
                    expired_at: None,
                    key: key.try_into()?,
                };
                self.authentication_keys.push(authentication_method);
            }
            EventType::CreateAgreementKey(key) => {}
            EventType::RevokeAssertionKey(kid) => {}
            EventType::RevokeAuthenticationKey(kid) => {}
            EventType::RevokeAgreementKey(kid) => {}
            EventType::SetProof(proof) => {}
        }
        Ok(())
    }
}
