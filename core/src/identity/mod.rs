use cid::Cid;

use crate::multi::{
    agreement_key::Idp2pAgreementKey, id::Idp2pCodec, key::Idp2pKey, key_digest::Idp2pKeyDigest,
    keypair::Idp2pKeypair,
};

use self::{error::IdentityError, state::IdentityState};

use std::collections::HashMap;

#[derive(PartialEq, Debug, Clone)]
pub struct IdEvents {
    pub assertion_key: Option<Idp2pKey>,
    pub authentication_key: Option<Idp2pKey>,
    pub agreement_key: Option<Idp2pAgreementKey>,
    pub proofs: HashMap<Vec<u8>, Vec<u8>>,
}

pub struct CreateIdentityInput {
    pub codec: Idp2pCodec,
    pub next_key_digest: Idp2pKeyDigest,
    pub recovery_key_digest: Idp2pKeyDigest,
    pub events: Option<IdEvents>,
}

pub struct ChangeInput {
    pub next_key_digest: Idp2pKeyDigest,
    pub recovery_key_digest: Idp2pKeyDigest,
    pub signer_keypair: Idp2pKeypair,
    pub events: IdEvents,
}

pub struct RecoverInput {
    pub next_key_digest: Idp2pKeyDigest,
    pub recovery_key_digest: Idp2pKeyDigest,
    pub signer_keypair: Idp2pKeypair,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Identity {
    pub id: Vec<u8>,
    pub microledger: Vec<u8>,
}

impl Identity {
    pub fn new(input: CreateIdentityInput) -> Result<Self, IdentityError> {
        match input.codec {
            Idp2pCodec::Protobuf => protobuf::factory::new(input),
            Idp2pCodec::Json => todo!(),
        }
    }

    pub fn verify(&self) -> Result<IdentityState, IdentityError> {
        match self.codec()? {
            Idp2pCodec::Protobuf => protobuf::verify::verify(self),
            Idp2pCodec::Json => todo!(),
        }
    }

    fn codec(&self) -> Result<Idp2pCodec, IdentityError> {
        let cid: Cid = self.id.to_vec().try_into()?;
        Ok(Idp2pCodec::try_from(cid.codec())?)
    }
}

pub mod doc;
pub mod error;
pub mod protobuf;
pub mod state;

#[cfg(test)]
mod tests {
    use crate::{identity::doc::IdentityDocument, multi::hash::Idp2pHash};

    use super::*;

    #[test]
    fn id_test() -> Result<(), IdentityError> {
        let did = create_did()?;
        let state = did.verify()?;
        let doc: IdentityDocument = state.into();
        eprintln!("{}", serde_json::to_string_pretty(&doc).unwrap());
        Ok(())
    }
    #[test]
    fn verify_invalid_id_test()-> Result<(), IdentityError> {
        let mut did = create_did()?;
        let mh = Idp2pHash::default().digest(&vec![]);
        did.id = Cid::new_v1(0x50, mh).to_bytes();
        let result = did.verify();
        let is_err = matches!(result, Err(IdentityError::InvalidId));
        assert!(is_err, "{:?}", result);
        Ok(())
    }

    fn create_did() -> Result<Identity, IdentityError>{
        let keypair = Idp2pKeypair::new_ed25519([0u8; 32])?;
        let input = CreateIdentityInput {
            codec: Idp2pCodec::Protobuf,
            next_key_digest: keypair.to_key().to_key_digest(),
            recovery_key_digest: keypair.to_key().to_key_digest(),
            events: Some(IdEvents{
                agreement_key: Some(keypair.to_agreement_key()),
                assertion_key: Some(keypair.to_key()),
                authentication_key: Some(keypair.to_key()),
                proofs: HashMap::new(),
            }),
        };
        Ok(Identity::new(input)?)
    }
    /*
    #[test]
    fn verify_test() {
        let ledger = create_microledger().0;
        let result = ledger.verify(&ledger.inception.get_id());
        assert!(result.is_ok(), "{:?}", result);
    }

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
