use idp2p_common::{
    cid::Cid,
    multi::{
        id::{Idp2pCid, Idp2pCodec},
        keypair::Idp2pKeypair,
    },
};

use self::{
    error::IdentityError,
    models::{ChangeType, IdEvent},
    state::IdentityState,
};
pub mod doc;
pub mod error;
pub mod json;
pub mod models;
pub mod protobuf;
pub mod state;
pub trait IdBehaviour {
    fn new(&self, input: CreateIdentityInput) -> Result<Identity, IdentityError>;
    fn change(&self, did: &mut Identity, input: ChangeInput) -> Result<bool, IdentityError>;
    fn verify(
        &self,
        did: &Identity,
        prev: Option<&Identity>,
    ) -> Result<IdentityState, IdentityError>;
}

pub struct CreateIdentityInput {
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
    // Bytes of inception cid(See https://github.com/multiformats/cid)
    pub id: Vec<u8>,
    // Microledger bytes(can be protobuf or json encoded)
    pub microledger: Vec<u8>,
}

impl Identity {
    /// Create a new identity
    pub fn new(codec: Idp2pCodec, input: CreateIdentityInput) -> Result<Self, IdentityError> {
        Self::resolve_behaviour(codec).new(input)
    }

    pub fn change(&mut self, input: ChangeInput) -> Result<bool, IdentityError> {
        Self::resolve_behaviour(self.codec()?).change(self, input)
    }

    /// Verify an identity and get state of identity
    pub fn verify(&self, prev: Option<&Identity>) -> Result<IdentityState, IdentityError> {
        Self::resolve_behaviour(self.codec()?).verify(self, prev)
    }

    fn resolve_behaviour(codec: Idp2pCodec) -> Box<dyn IdBehaviour> {
        match codec {
            Idp2pCodec::Protobuf => Box::new(protobuf::behaviour::ProtoIdentityBehavior),
            Idp2pCodec::Json => Box::new(json::behaviour::JsonIdentityBehavior),
        }
    }

    fn codec(&self) -> Result<Idp2pCodec, IdentityError> {
        let cid = Cid::from_bytes(&self.id)?;
        Ok(Idp2pCodec::try_from(cid.codec())?)
    }
}

#[cfg(test)]
mod tests {
    use idp2p_common::multi::{hash::Idp2pHash, keypair::Idp2pKeypair};

    use super::{
        models::{ChangeType, IdEvent},
        *,
    };
    use crate::identity::doc::IdentityDocument;

    #[test]
    fn id_test() -> Result<(), IdentityError> {
        let did = create_did()?;
        let state = did.verify(None)?;
        /*let doc: IdentityDocument = state.into();
        eprintln!("{}", serde_json::to_string_pretty(&doc).unwrap());*/
        Ok(())
    }
    #[test]
    fn verify_invalid_id_test() -> Result<(), IdentityError> {
        let mut did = create_did()?;
        let mh = Idp2pHash::default().digest(&vec![]);
        did.id = Cid::new_v1(0x51, mh).to_bytes();
        let result = did.verify(None);
        let is_err = matches!(result, Err(IdentityError::Idp2pMultiError(_)));
        assert!(is_err, "{:?}", result);
        Ok(())
    }

    fn create_did() -> Result<Identity, IdentityError> {
        let keypair = Idp2pKeypair::new_ed25519([0u8; 32])?;
        let input = CreateIdentityInput {
            next_key_digest: keypair.to_key().to_key_digest(),
            recovery_key_digest: keypair.to_key().to_key_digest(),
            events: vec![IdEvent::CreateAuthenticationKey {
                id: vec![1],
                key: keypair.to_key().to_bytes(),
            }],
        };
        let key = keypair.to_key();
        let mut did = Identity::new(Idp2pCodec::Json, input)?;
        for i in 2..10 {
            let change_input = ChangeInput {
                next_key_digest: key.to_key_digest(),
                signer_keypair: keypair.clone(),
                change: ChangeType::AddEvents {
                    events: vec![IdEvent::CreateAuthenticationKey {
                        id: vec![i],
                        key: key.to_bytes(),
                    }],
                },
            };
            did.change(change_input).unwrap();
        }
        eprintln!("length: {}", did.microledger.len());
        Ok(did)
    }
    /*
        #[test]
        fn verify_test() {
            let ledger = create_microledger().0;
            let result = ledger.verify(&ledger.inception.get_id());
            assert!(result.is_ok(), "{:?}", result);
        }
    Some(IdEvents {
                    agreement_key: Some(keypair.to_agreement_key()),
                    assertion_key: Some(keypair.to_key()),
                    authentication_key: Some(keypair.to_key()),
                    proofs: HashMap::new(),
                })
        #[test]
        fn verify_invalid_id_test() {
            let ledger = create_microledger().0;
            let result = ledger.verify("1");
            let is_err = matches!(result, Err(crate::IdentityError::InvalidId));
            assert!(is_err, "{:?}", result);
        }

        #[test]
        fn verify_valid_test() {
            let (mut ledger, secret) = create_microledger();
            let id = ledger.inception.get_id();
            let set_proof = EventLogChange::SetProof(ProofStatement {
                key: vec![1],
                value: vec![1],
            });
            let ver_method = VerificationMethod {
                id: id.clone(),
                controller: id.clone(),
                typ: ED25519.to_string(),
                bytes: secret.to_publickey().to_vec(),
            };
            let set_assertion = EventLogChange::SetAssertionKey {
                verification_method: ver_method.clone(),
            };
            let set_authentication = EventLogChange::SetAuthenticationKey {
                verification_method: ver_method.clone(),
            };
            let set_agreement = EventLogChange::SetAgreementKey {
                verification_method: ver_method.clone(),
            };
            let change = vec![
                set_proof,
                set_assertion.clone(),
                set_authentication,
                set_agreement,
            ];
            let signer = secret.to_publickey();
            let payload = ledger.create_event(&signer, &secret.to_publickey_digest().unwrap(), change);
            let proof = secret.sign(&payload);
            ledger.save_event(payload, &proof);
            let change = vec![set_assertion];
            let payload = ledger.create_event(&signer, &signer, change);
            let proof = secret.sign(&payload);
            ledger.save_event(payload, &proof);
            let result = ledger.verify(&id);
            assert!(result.is_ok());
        }
        #[test]
        fn verify_invalid_previous_test() {
            let (mut ledger, secret) = create_microledger();
            let id = ledger.inception.get_id();
            let set_change = EventLogChange::SetProof(ProofStatement {
                key: vec![],
                value: vec![],
            });
            let change = vec![set_change];
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
            let set_change = EventLogChange::SetProof(ProofStatement {
                key: vec![],
                value: vec![],
            });
            let change = vec![set_change];
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
        fn verify_invalid_signer_test() {
            let (mut ledger, secret) = create_microledger();
            let id = ledger.inception.get_id();
            let set_change = EventLogChange::SetProof(ProofStatement {
                key: vec![],
                value: vec![],
            });
            let change = vec![set_change];
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

        fn create_microledger() -> (MicroLedger, idp2p_common::secret::EdSecret) {
            let secret_str = "bd6yg2qeifnixj4x3z2fclp5wd3i6ysjlfkxewqqt2thie6lfnkma";
            let secret = idp2p_common::secret::EdSecret::from_str(secret_str).unwrap();
            let d = secret.to_publickey_digest().unwrap();
            let ledger = MicroLedger::new(&d, &d);
            (ledger, secret)
        }*/
}
