use crate::{
    error::Idp2pError,
    idp2p_proto::{self},
};
use idp2p_common::multi::{
    id::Idp2pId, ledgerkey::Idp2pLedgerKeypair, verification::ed25519::Ed25519Keypair,
};
use prost::Message;

#[derive(PartialEq, Debug, Clone)]
pub struct IdentityState {
    pub id: Vec<u8>,
    pub latest_event_id: Vec<u8>,
    pub owner_next_pk_hash: Vec<u8>,
    pub root_next_pk_hash: Vec<u8>,
    pub proof: Vec<u8>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Identity {
    pub id: Vec<u8>,
    pub microledger: Vec<u8>,
}

impl Identity {
    pub fn new(owner_pk_hash: &[u8], root_pk_hash: &[u8]) -> Result<Identity, Idp2pError> {
        let inception = idp2p_proto::Idp2pInception {
            next_pk_hash: owner_pk_hash.to_vec(),
            rec_next_pk_hash: root_pk_hash.to_vec()
        }
        .encode_to_vec();
        let id = Idp2pId::new(1, &inception);
        let microledger = idp2p_proto::Idp2pMicroledger {
            inception: inception,
            events: vec![],
            proofs: vec![]
        };
        Ok(Identity {
            id: id.to_bytes(),
            microledger: microledger.encode_to_vec(),
        })
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Identity, Idp2pError> {
        todo!()
    }

    pub fn mutate(&mut self, proof: &[u8]) -> Result<(), Idp2pError> {
        let owner_keypair = Idp2pLedgerKeypair::Ed25519(Ed25519Keypair::generate());
        let id = Idp2pId::from_bytes(&self.id)?;
        todo!()
    }

    /*/// Verify an identity and get state of identity
    pub fn verify(&self) -> Result<IdentityState, Idp2pError> {
        let id = Idp2pId::from_bytes(&self.id)?;
        let microledger = idp2p_proto::Idp2pMicroledger::decode(&*self.microledger)?;
        id.ensure(&microledger.inception)?;
        // Decode inception bytes of microledger
        let inception = idp2p_proto::Idp2pInception::decode(&*microledger.inception)?;
        // Init current state to handle events
        let mut state = IdentityState {
            id: id.to_bytes(),
            latest_event_id: id.to_bytes(),
            owner_next_pk_hash: inception.owner_next_pk_hash.clone(),
            root_next_pk_hash: inception.root_next_pk_hash.clone(),
            proofs: vec![],
        };
        for event in microledger.events {
            let event_id = Idp2pId::from_bytes(&event.id)?;
            event_id.ensure(&event.payload)?;
            let payload = idp2p_proto::Idp2pEventPayload::decode(event.payload.as_slice())?;
            if payload.previous != state.latest_event_id {
                return Err(Idp2pError::InvalidPreviousEventLog);
            }
            let event_kind = payload
                .event_kind
                .ok_or(Idp2pError::RequiredField("change".to_string()))?;
            use idp2p_proto::idp2p_event_payload::EventKind;
            match event_kind {
                EventKind::Mutation(mutation) => {
                    let owner_next_key_hash =
                        Idp2pLedgerPublicDigest::from_multi_bytes(&state.owner_next_pk_hash)?;
                    owner_next_key_hash.ensure_public(&mutation.owner_pk)?;
                    let signer =
                        Idp2pLedgerPublicKey::new(owner_next_key_hash.code(), &mutation.owner_pk)?;
                    signer.verify(&event.payload, &event.signature)?;
                    state.owner_next_pk_hash = mutation.owner_next_pk_hash;
                    state.proofs.push(mutation.proof);
                }
                EventKind::Recover(recover) => {
                    let root_next_key_hash =
                        Idp2pLedgerPublicDigest::from_multi_bytes(&state.root_next_pk_hash)?;
                    root_next_key_hash.ensure_public(&recover.root_pk)?;
                    let signer =
                        Idp2pLedgerPublicKey::new(root_next_key_hash.code(), &recover.root_pk)?;
                    signer.verify(&event.payload, &event.signature)?;
                    state.root_next_pk_hash = recover.root_next_pk_hash;
                }
            }
        }
        Ok(state)
    }*/
}

#[cfg(test)]
mod tests {
    //use idp2p_common::multi::base::Idp2pBase;

    use super::*;
    /*let owner_keypair = Idp2pLedgerKeypair::Ed25519(Ed25519Keypair::generate());
    let root_keypair = Idp2pLedgerKeypair::Winternitz(WinternitzKeypair::generate());
    let inception = idp2p_proto::Idp2pInception{
    owner_next_pk_hash: owner_keypair.to_public_key().to_digest()?.to_multi_bytes(),
    root_next_pk_hash: root_keypair.to_public_key().to_digest()?.to_multi_bytes()
    };*/
    #[test]
    fn id_test() -> Result<(), Idp2pError> {
        /*let secret_str = "bd6yg2qeifnixj4x3z2fclp5wd3i6ysjlfkxewqqt2thie6lfnkma";

        let keypair = Idp2pLedgerKeypair::Ed25519(Ed25519Keypair::from_secret_bytes(
            Idp2pBase::decode_sized::<32>(secret_str)?,
        ));
        let expected_id = "z3Yyn15YzHQLhEz2a1VvF2q73picqNiUyjqLVfUSMcDyuKkq1fW";
        let did = Identity::new(
            &keypair.to_public_key().to_digest()?.to_multi_bytes(),
            &keypair.to_public_key().to_digest()?.to_multi_bytes(),
        )?;
        assert_eq!(Idp2pBase::default().encode(&did.id), expected_id);*/
        Ok(())
    }

    /*#[test]
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
        let original_did = did.clone();
        let input = ChangeInput {
            next_key_digest: keypair.to_public_key().to_digest()?.to_multi_bytes(),
            signer_keypair: keypair,
            change: ChangeType::AddEvents { events: vec![] },
        };
        did.change(input)?;
        let result = original_did.verify(Some(&did));
        let is_err = matches!(result, Err(Idp2pError::InvalidPrevious));
        assert!(is_err, "{:?}", result);
        Ok(())
    }*/
}
