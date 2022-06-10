use crate::identity::{
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
}
/*
  use capnp::serialize_packed;
    let v: Vec<u8> = vec![];
    let message_reader =
        serialize_packed::read_message(v.as_slice(), ::capnp::message::ReaderOptions::new()).unwrap();
    let microledger = message_reader.get_root::<microledger::Reader>().unwrap();
    let inception_bytes = microledger.reborrow().get_inception().unwrap();
    let inception = message_reader.get_root::<identity_inception::Reader>().unwrap();
    for event in inception.reborrow().get_events().iter(){
         for e in event.reborrow().iter(){
*/
/*impl IdentityStateEventHandler<> for IdentityState {
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
}*/
