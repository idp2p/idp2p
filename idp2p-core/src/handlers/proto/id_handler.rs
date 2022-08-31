use std::collections::HashMap;

use idp2p_common::{
    chrono::Utc,
    multi::{id::Idp2pId, ledgerkey::Idp2pLedgerPublicKey},
};
use prost::Message;

use crate::{
    error::Idp2pError,
    handlers::proto::mapper::EventLogResolver,
    id_state::{IdentityState, IdentityStateEventHandler},
    identity::{ChangeInput, ChangeType, CreateIdentityInput, IdEvent, Identity, IdentityHandler},
    idp2p_proto,
};

pub struct ProtoIdentityHandler;

impl IdentityHandler for ProtoIdentityHandler {
    fn new(&self, input: CreateIdentityInput) -> Result<Identity, Idp2pError> {
        let mut inception = idp2p_proto::IdentityInception {
            timestamp: input.timestamp,
            next_key_digest: input.next_key_digest,
            recovery_key_digest: input.recovery_key_digest,
            events: vec![],
        };
        for id_event in input.events {
            match id_event {
                IdEvent::RevokeAssertionKey(_)
                | IdEvent::RevokeAuthenticationKey(_)
                | IdEvent::RevokeAgreementKey(_) => {
                    return Err(Idp2pError::Other);
                }
                _ => inception.events.push(id_event.into()),
            }
        }

        let inception_bytes = inception.encode_to_vec();
        let id = create_id(&inception_bytes);
        let microledger = idp2p_proto::Microledger {
            inception: inception_bytes,
            event_logs: vec![],
        };
        let did = Identity {
            id: id,
            microledger: microledger.encode_to_vec(),
        };
        Ok(did)
    }

    fn change(&self, did: &mut Identity, input: ChangeInput) -> Result<bool, Idp2pError> {
        use idp2p_proto::event_log_payload::{Change, IdentityEvents};
        let state = self.verify(did, None)?;
        let signer_key: Idp2pLedgerPublicKey = input.signer_keypair.to_public_key();
        let mut payload = idp2p_proto::EventLogPayload {
            previous: state.last_event_id,
            signer_key: signer_key.to_bytes(), // Raw public bytes because it is implicitly decided with digest
            next_key_digest: input.next_key_digest,
            timestamp: Utc::now().timestamp(),
            change: None,
        };

        match input.change {
            ChangeType::AddEvents { events } => {
                macro_rules! validate_new_key {
                    ($ks: ident, $kid: expr) => {{
                        if state.$ks.iter().any(|k| k.id == $kid) {
                            return Err(Idp2pError::InvalidCreateKey);
                        }
                    }};
                }
                macro_rules! validate_revoke_key {
                    ($ks: ident, $kid: expr) => {{
                        if state.$ks.iter().any(|k| k.id == $kid) {
                            return Err(Idp2pError::InvalidRevokeKey);
                        }
                    }};
                }
                let mut id_events: Vec<idp2p_proto::IdentityEvent> = vec![];
                for event in events {
                    match &event {
                        IdEvent::CreateAssertionKey { id, multi_bytes: _ } => {
                            validate_new_key!(assertion_keys, *id)
                        }
                        IdEvent::CreateAuthenticationKey { id, multi_bytes: _ } => {
                            validate_new_key!(authentication_keys, *id)
                        }
                        IdEvent::CreateAgreementKey { id, multi_bytes: _ } => {
                            validate_new_key!(agreement_keys, *id)
                        }
                        IdEvent::RevokeAssertionKey(kid) => {
                            validate_revoke_key!(assertion_keys, *kid)
                        }
                        IdEvent::RevokeAuthenticationKey(kid) => {
                            validate_revoke_key!(authentication_keys, *kid)
                        }
                        IdEvent::RevokeAgreementKey(kid) => {
                            validate_revoke_key!(agreement_keys, *kid)
                        }
                        _ => {}
                    }
                    id_events.push(event.into());
                }
                payload.change = Some(Change::Events(IdentityEvents { events: id_events }));
            }
            ChangeType::Recover(key_digest) => {
                payload.change = Some(Change::Recover(key_digest));
            }
        }
        let payload_bytes = payload.encode_to_vec();
        let proof = input.signer_keypair.sign(&payload_bytes)?;
        let event_log = idp2p_proto::EventLog {
            id: create_id(&payload_bytes),
            payload: payload_bytes,
            proof: proof,
        };
        let mut microledger = idp2p_proto::Microledger::decode(&*did.microledger)?;
        microledger.event_logs.push(event_log);
        did.microledger = microledger.encode_to_vec();
        Ok(true)
    }

    fn verify(
        &self,
        identity: &Identity,
        prev: Option<&Identity>,
    ) -> Result<IdentityState, Idp2pError> {
        let microledger = idp2p_proto::Microledger::decode(&*identity.microledger)?;
        let id = Idp2pId::from_bytes(&identity.id)?;
        // Check cid is produced with inception
        id.ensure(&microledger.inception)?;
        // If there is previous id check if it is base of that id
        if let Some(prev) = prev {
            is_valid_prev(&microledger, prev)?;
        }
        // Decode inception bytes of microledger
        let inception = idp2p_proto::IdentityInception::decode(&*microledger.inception)?;
        // Init current state to handle events
        let mut state = IdentityState {
            id: id.to_bytes(),
            last_event_id: id.to_bytes(), // First event is the id hash
            next_key_digest: inception.next_key_digest,
            recovery_key_digest: inception.recovery_key_digest,
            assertion_keys: vec![],
            authentication_keys: vec![],
            agreement_keys: vec![],
            proofs: HashMap::new(),
        };
        // Handle initial events
        for event in inception.events {
            let event = event
                .event_type
                .ok_or(Idp2pError::RequiredField("event_type".to_string()))?;
            state.handle_event(inception.timestamp, event)?;
        }
        use idp2p_proto::event_log_payload::Change;
        for log in microledger.event_logs {
            let log_id = Idp2pId::from_bytes(&log.id)?;
            log_id.ensure(&log.payload)?;
            let payload = log.try_resolve_payload()?;
            // Previous event_id should match with last_event_id of state.
            // Because all identity events point previous event.
            // First event points to inception event
            if payload.previous != state.last_event_id {
                return Err(Idp2pError::InvalidPreviousEventLog);
            }
            let change = payload
                .change
                .ok_or(Idp2pError::RequiredField("change".to_string()))?;
            match change {
                Change::Recover(key_digest) => {
                    let signer = state.next_rec_key(&payload.signer_key)?;
                    signer.verify(&log.payload, &log.proof)?;
                    state.recovery_key_digest = key_digest;
                }
                Change::Events(events) => {
                    let signer = state.next_signer_key(&payload.signer_key)?;
                    signer.verify(&log.payload, &log.proof)?;
                    for event in events.events {
                        let event = event
                            .event_type
                            .ok_or(Idp2pError::RequiredField("event_type".to_string()))?;
                        state.handle_event(payload.timestamp, event)?;
                    }
                }
            }
            state.next_key_digest = payload.next_key_digest;
            state.last_event_id = log.id;
        }
        Ok(state)
    }
}

fn create_id(content: &[u8]) -> Vec<u8> {
    Idp2pId::new(0, &content).to_bytes()
}
fn is_valid_prev(c: &idp2p_proto::Microledger, prev: &Identity) -> Result<bool, Idp2pError> {
    let prev_ml = idp2p_proto::Microledger::decode(&*prev.microledger)?;
    for (i, log) in prev_ml.event_logs.iter().enumerate() {
        if log.id != c.event_logs[i].id {
            return Err(Idp2pError::InvalidPrevious);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use idp2p_common::{
        multi::{base::Idp2pBase, error::Idp2pMultiError, ledgerkey::Idp2pLedgerKeypair},
        random::create_random,
    };

    use super::*;
    fn create() -> Result<(Identity, Idp2pLedgerKeypair), Idp2pError> {
        let keypair = Idp2pLedgerKeypair::new_ed25519(create_random::<32>())?;
        let input = CreateIdentityInput {
            timestamp: Utc::now().timestamp(),
            next_key_digest: keypair.to_public_key().to_digest()?.to_multi_bytes(),
            recovery_key_digest: keypair.to_public_key().to_digest()?.to_multi_bytes(),
            events: vec![],
        };
        Ok((ProtoIdentityHandler {}.new(input)?, keypair))
    }

    #[test]
    fn id_test() -> Result<(), Idp2pError> {
        let secret_str = "bd6yg2qeifnixj4x3z2fclp5wd3i6ysjlfkxewqqt2thie6lfnkma";
        let keypair = Idp2pLedgerKeypair::new_ed25519(Idp2pBase::decode_sized::<32>(secret_str)?)?;
        let expected_id = "z3YygDRExrCXjGa8PEMeTWWTZMCFtVHwa84KtnQp6Uqb1YMCJUU";
        let input = CreateIdentityInput {
            timestamp: 0,
            next_key_digest: keypair.to_public_key().to_digest()?.to_multi_bytes(),
            recovery_key_digest: keypair.to_public_key().to_digest()?.to_multi_bytes(),
            events: vec![],
        };
        let did = ProtoIdentityHandler {}.new(input)?;
        assert_eq!(Idp2pBase::default().encode(&did.id), expected_id);
        Ok(())
    }

    #[test]
    fn verify_ok_test() -> Result<(), Idp2pError> {
        let (did, _) = create()?;
        let result = did.verify(None);
        assert!(result.is_ok(), "{:?}", result);
        Ok(())
    }

    #[test]
    fn verify_invalid_id_test() -> Result<(), Idp2pError> {
        let (mut did, _) = create()?;
        let l = did.id.len() - 1;
        did.id[l] = 1u8;
        let result = did.verify(None);
        let is_err = matches!(
            result,
            Err(Idp2pError::Idp2pMultiError(Idp2pMultiError::InvalidCid))
        );
        assert!(is_err, "{:?}", result);
        Ok(())
    }

    #[test]
    fn verify_invalid_previous_test() -> Result<(), Idp2pError> {
        let (mut did, keypair) = create()?;
        let input = ChangeInput {
            next_key_digest: keypair.to_public_key().to_digest()?.to_multi_bytes(),
            signer_keypair: keypair,
            change: ChangeType::AddEvents { events: vec![] },
        };
        did.change(input.clone())?;
        //did.change(input)?;
        let result = did.verify(None);
        assert!(result.is_ok(), "{:?}", result);
        /*let id = ledger.inception.get_id();
        let change = EventLogChange::SetDocument(DocumentDigest { value: vec![] });
        let signer = secret.to_publickey();
        let payload = ledger.create_event(&signer, &signer, change);
        let proof = secret.sign(&payload);
        ledger.save_event(payload, &proof);
        ledger.events[0].payload.previous = "1".to_owned();
        let result = ledger.verify(&id);
        let is_err = matches!(result, Err(crate::IdentityError::InvalidPrevious));
        assert!(is_err, "{:?}", result);*/
        Ok(())
    }

    /*#[test]
    fn verify_invalid_signature_test() {
        let (mut ledger, secret) = create_microledger();
        let id = ledger.inception.get_id();
        let change = EventLogChange::SetDocument(DocumentDigest { value: vec![] });
        let signer = secret.to_publickey();
        let payload = ledger.create_event(&signer, &signer, change);
        let proof = secret.sign(&payload);
        ledger.save_event(payload, &proof);
        ledger.events[0].proof = vec![0; 64];
        let result = ledger.verify(&id);
        let is_err = matches!(result, Err(crate::IdentityError::InvalidEventSignature));
        assert!(is_err, "{:?}", result);
    }

    #[test]
    fn verify_set_doc_invalid_signer_test() {
        let (mut ledger, secret) = create_microledger();
        let id = ledger.inception.get_id();
        let change = EventLogChange::SetDocument(DocumentDigest { value: vec![] });
        let signer = secret.to_publickey();
        let payload = ledger.create_event(&signer, &signer, change);
        let proof = secret.sign(&payload);
        ledger.save_event(payload, &proof);
        let new_secret = EdSecret::new();
        let new_ed_key = new_secret.to_publickey();
        ledger.events[0].payload.signer_key = new_ed_key.to_vec();
        ledger.events[0].proof = new_secret.sign(&ledger.events[0].payload).to_vec();
        let result = ledger.verify(&id);
        let is_err = matches!(result, Err(crate::IdentityError::InvalidSigner));
        assert!(is_err, "{:?}", result);
    }

    #[test]
    fn verify_set_proof_invalid_signer_test() {
        let (mut ledger, secret) = create_microledger();
        let id = ledger.inception.get_id();
        let change = EventLogChange::SetProof(ProofStatement {
            key: vec![],
            value: vec![],
        });
        let signer = secret.to_publickey();
        let payload = ledger.create_event(&signer, &signer, change);
        let proof = secret.sign(&payload);
        ledger.save_event(payload, &proof);
        let new_secret = EdSecret::new();
        let new_ed_key = new_secret.to_publickey();
        ledger.events[0].payload.signer_key = new_ed_key.to_vec();
        ledger.events[0].proof = new_secret.sign(&ledger.events[0].payload).to_vec();
        let result = ledger.verify(&id);
        let is_err = matches!(result, Err(crate::IdentityError::InvalidSigner));
        assert!(is_err, "{:?}", result);
    }

    #[test]
    fn verify_recovery_invalid_signer_test() {
        let (mut ledger, secret) = create_microledger();
        let id = ledger.inception.get_id();
        let signer = secret.to_publickey();
        let rec = RecoverStatement {
            key_type: ED25519.to_owned(),
            recovery_key_digest: hash(&signer),
        };
        let change = EventLogChange::Recover(rec);
        let payload = ledger.create_event(&signer, &signer, change);
        let proof = secret.sign(&payload);
        ledger.save_event(payload, &proof);
        let new_secret = EdSecret::new();
        let new_ed_key = new_secret.to_publickey();
        ledger.events[0].payload.signer_key = new_ed_key.to_vec();
        ledger.events[0].proof = new_secret.sign(&ledger.events[0].payload).to_vec();
        let result = ledger.verify(&id);
        let is_err = matches!(result, Err(crate::IdentityError::InvalidSigner));
        assert!(is_err, "{:?}", result);
    }

    fn create_microledger() -> (MicroLedger, idp2p_common::ed_secret::EdSecret) {
        let secret_str = "bd6yg2qeifnixj4x3z2fclp5wd3i6ysjlfkxewqqt2thie6lfnkma";
        let secret = idp2p_common::ed_secret::EdSecret::from_str(secret_str).unwrap();
        let d = secret.to_publickey_digest().unwrap();
        let ledger = MicroLedger::new(&d, &d);
        (ledger, secret)
    }*/
}
