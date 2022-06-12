/*use crate::identity::{
    error::IdentityError,
    state::{AgreementKeyState, IdentityState, IdentityStateEventHandler, KeyState, ProofState},
};
use crate::identity_capnp::{event_log, identity_event, identity_inception, microledger};
use idp2p_common::multi::{agreement_key::Idp2pAgreementKey, key::Idp2pKey};


fn handle_event(state: &mut IdentityState, timestamp: i64, event: identity_event::Reader) {
    match event.which() {
        Ok(identity_event::CreateAgreementKey(Ok(key))) => {
            let previous_key = state.assertion_keys.last_mut();
            if let Some(previous_key) = previous_key {
                previous_key.expired_at = Some(timestamp);
            }
            let assertion_method = KeyState {
                id: key.get_id().unwrap().to_vec(),
                valid_at: timestamp,
                expired_at: None,
                key: key.get_value().unwrap().to_vec(),
            };
            state.assertion_keys.push(assertion_method);
        }
        Err(_) => todo!(),
        _ => todo!(),
    }
}*/
