use std::collections::HashMap;

use super::{
    core::{CreateIdentityInput, IdentityBehaviour, IdentityEvent, IdentityState},
    did_doc::IdentityDocument,
    idp2p_proto::{
        self, event_log_payload::EventLogChange, identity_event::IdentityEventType,
        EventLogPayload, Identity, IdentityInception,
    },
};
use idp2p_common::{
    anyhow::Result, generate_cid, key::Idp2pKey, key_digest::Idp2pKeyDigest, secret::Idp2pSecret,
    Idp2pCodec, Idp2pHash,
};
use prost::Message;

impl IdentityBehaviour for Identity {
    type IdType = Identity;

    fn create(input: CreateIdentityInput) -> Result<Self::IdType> {
        todo!()
    }

    fn verify(&self) -> Result<IdentityState> {
        /*let inception_bytes = self.microledger.as_ref().expect("").inception.clone();
        let expected_id = generate_cid(&inception_bytes, Idp2pCodec::Protobuf, Idp2pHash::Sha256);
        assert_eq!(expected_id, self.id);
        let inception = IdentityInception::decode(&*inception_bytes)?;
        let inc_next_key_digest = inception.next_key_digest.expect("Next key digest required");
        let inc_recovery_key_digest = inception
            .recovery_key_digest
            .expect("Recovery key digest required");
        let state = IdentityState {
            event_id: self.id,
            next_key_digest: Idp2pKeyDigest::from_bytes(
                inc_next_key_digest.alg as u8,
                &inc_next_key_digest.digest,
            )?,
            recovery_key_digest: Idp2pKeyDigest::from_bytes(
                inc_recovery_key_digest.alg as u8,
                &inc_recovery_key_digest.digest,
            )?,
            assertion_keys: vec![],
            authentication_key: None,
            agreement_key: None,
            proofs: HashMap::new(),
        };
        for event_log in self.microledger.as_ref().expect("msg").event_logs.iter() {
            let payload = EventLogPayload::decode(&*event_log.payload).expect("msg");
            let verify_key =
                Idp2pKey::from_bytes(state.next_key_digest.get_alg(), &payload.signer_key)?;
        }*/
        todo!()
    }

    fn recover(&mut self, signer: Idp2pSecret, rec_digest: Idp2pKeyDigest) -> Result<()> {
        todo!()
    }

    fn add_events(&mut self, signer: Idp2pSecret, events: Vec<IdentityEvent>) -> Result<()> {
        todo!()
    }

    fn to_document(&self) -> Result<IdentityDocument> {
        todo!()
    }
}
