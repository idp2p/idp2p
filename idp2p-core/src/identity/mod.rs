use idp2p_common::{chrono::Utc, multi::{keypair::Idp2pKeypair}, random::create_random};

use crate::{store::IdSecret};

use self::{
    error::IdentityError,
    handler::{id_handler::ProtoIdentityHandler, msg_handler::ProtoMessageHandler},
    models::{ChangeType, IdEvent},
    state::IdentityState, message::{MessageHandler, IdMessage},
};
pub mod doc;
pub mod error;
pub mod handler;
pub mod models;
pub mod state;
pub mod message;

pub struct CreateIdentityInput {
    pub timestamp: i64,
    // Next key digest(multikey digest)
    pub next_key_digest: Vec<u8>,
    // Recovery key digest(multikey digest)
    pub recovery_key_digest: Vec<u8>,
    pub events: Vec<IdEvent>,
}

#[derive(Debug)]
pub struct ChangeInput {
    pub next_key_digest: Vec<u8>,
    pub signer_keypair: Idp2pKeypair,
    pub change: ChangeType,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Identity {
    // Bytes of inception id(idp2p multi id)
    pub id: Vec<u8>,
    // Microledger bytes(can be protobuf, json ... encoded)
    pub microledger: Vec<u8>,
}

pub trait IdentityHandler {
    fn new(&self, input: CreateIdentityInput) -> Result<Identity, IdentityError>;
    fn change(&self, did: &mut Identity, input: ChangeInput) -> Result<bool, IdentityError>;
    fn verify(
        &self,
        did: &Identity,
        prev: Option<&Identity>,
    ) -> Result<IdentityState, IdentityError>;
}

impl Identity {
    pub fn create() -> Result<Self, IdentityError> {
        let keypair = Idp2pKeypair::from_ed_secret(&create_random::<32>())?;
        let input = CreateIdentityInput {
            timestamp: Utc::now().timestamp(),
            next_key_digest: keypair.to_key().to_key_digest(),
            recovery_key_digest: keypair.to_key().to_key_digest(),
            events: vec![],
        };
        ProtoIdentityHandler {}.new(input)
    }

    pub fn new(input: CreateIdentityInput) -> Result<Self, IdentityError> {
        ProtoIdentityHandler {}.new(input)
    }

    pub fn change(&mut self, input: ChangeInput) -> Result<bool, IdentityError> {
        ProtoIdentityHandler {}.change(self, input)
    }

    /// Verify an identity and get state of identity
    pub fn verify(&self, prev: Option<&Identity>) -> Result<IdentityState, IdentityError> {
        ProtoIdentityHandler {}.verify(self, prev)
    }

    pub fn seal_msg(&self, to: IdentityState, body: &[u8]) -> Result<Vec<u8>, IdentityError>{
        let from_state = self.verify(None)?;
        Ok(vec![])
    }

    pub fn decode_msg(&self, msg: &[u8], agreement_secret: IdSecret) -> Result<IdMessage, IdentityError>{
        ProtoMessageHandler {}.decode_msg(msg, agreement_secret)
    }

}

#[cfg(test)]
mod tests {
    use idp2p_common::multi::{base::Idp2pBase, error::Idp2pMultiError};

    use super::*;

    #[test]
    fn id_test() -> Result<(), IdentityError> {
        let secret_str = "bd6yg2qeifnixj4x3z2fclp5wd3i6ysjlfkxewqqt2thie6lfnkma";
        let keypair = Idp2pKeypair::from_ed_secret(&Idp2pBase::decode(secret_str)?)?;
        let expected_id = "z3YygDRExrCXjGa8PEMeTWWTZMCFtVHwa84KtnQp6Uqb1YMCJUU";
        let input = CreateIdentityInput {
            timestamp: 0,
            next_key_digest: keypair.to_key().to_key_digest(),
            recovery_key_digest: keypair.to_key().to_key_digest(),
            events: vec![],
        };
        let did = Identity::new(input)?;
        assert_eq!(Idp2pBase::default().encode(&did.id), expected_id);
        Ok(())
    }

    #[test]
    fn verify_test() -> Result<(), IdentityError> {
        let did = Identity::create()?;
        let result = did.verify(None);
        assert!(result.is_ok(), "{:?}", result);
        eprintln!("{:?}", result.unwrap());
        Ok(())
    }

    #[test]
    fn verify_invalid_id_test()-> Result<(), IdentityError> {
        let mut did = Identity::create()?;
        let l = did.id.len()- 1;
        did.id[l] = 1u8;
        let result = did.verify(None);
        let is_err = matches!(result, Err(IdentityError::Idp2pMultiError(Idp2pMultiError::InvalidCid)));
        assert!(is_err, "{:?}", result);
        Ok(())
    }

    /*#[test]
    fn verify_invalid_previous_test() {
        let (mut ledger, secret) = create_microledger();
        let id = ledger.inception.get_id();
        let change = EventLogChange::SetDocument(DocumentDigest { value: vec![] });
        let signer = secret.to_publickey();
        let payload = ledger.create_event(&signer, &signer, change);
        let proof = secret.sign(&payload);
        ledger.save_event(payload, &proof);
        ledger.events[0].payload.previous = "1".to_owned();
        let result = ledger.verify(&id);
        let is_err = matches!(result, Err(crate::IdentityError::InvalidPrevious));
        assert!(is_err, "{:?}", result);
    }

    #[test]
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
