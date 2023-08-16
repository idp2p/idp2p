use crate::{
    error::Idp2pError,
    idp2p_proto::{
        idp2p_event_payload::EventKind, Idp2pEvent, Idp2pEventPayload, Idp2pInception,
        Idp2pMicroledger,
    },
};
use idp2p_common::multi::{
    hash::Idp2pMultiHash,
    id::Idp2pId,
    ledgerkey::{Idp2pLedgerPublicHash, Idp2pLedgerPublicKey},
};
use prost::Message;

#[derive(PartialEq, Debug, Clone)]
pub struct IdentityState {
    pub id: Vec<u8>,
    pub m: u32,
    pub r: u32,
    pub n: u32,
    pub key_hashs: Vec<Vec<u8>>,
    pub event_id: Vec<u8>,
    pub sdt_proof: Vec<u8>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Identity {
    pub id: Vec<u8>,
    pub microledger: Vec<u8>,
}

impl Identity {
    pub fn new(inception: Idp2pInception) -> Result<Identity, Idp2pError> {
        let microledger = Idp2pMicroledger {
            inception: inception.encode_to_vec(),
            events: vec![],
        };
        let mh = Idp2pMultiHash::new(&microledger.inception)?;
        Ok(Identity {
            id: Idp2pId::Identity(mh.to_bytes()).to_bytes()?,
            microledger: microledger.encode_to_vec(),
        })
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Identity, Idp2pError> {
        let microledger: Idp2pMicroledger = Idp2pMicroledger::decode(bytes)?;

        let mh = Idp2pMultiHash::new(&microledger.inception)?;
        Ok(Identity {
            id: Idp2pId::Identity(mh.to_bytes()).to_bytes()?,
            microledger: bytes.to_vec(),
        })
    }

    pub fn prepare_mutation(&self, sdt_proof: &[u8]) -> Result<Vec<u8>, Idp2pError> {
        let state = self.verify()?;
        let event_kind = EventKind::Mutation(sdt_proof.to_vec());
        let payload = Idp2pEventPayload {
            previous: state.event_id,
            event_kind: Some(event_kind),
        };
        Ok(payload.encode_to_vec())
    }

    pub fn prepare_recover(&self, recover: Idp2pInception) -> Result<Vec<u8>, Idp2pError> {
        let state = self.verify()?;
        let event_kind = EventKind::Recover(recover);
        let payload = Idp2pEventPayload {
            previous: state.event_id,
            event_kind: Some(event_kind),
        };
        Ok(payload.encode_to_vec())
    }

    pub fn push_event(&mut self, event: Idp2pEvent) -> Result<(), Idp2pError> {
        let mut microledger: Idp2pMicroledger = Idp2pMicroledger::decode(&*self.microledger)?;
        microledger.events.push(event);
        self.microledger = microledger.encode_to_vec();
        Ok(())
    }

    pub fn verify(&self) -> Result<IdentityState, Idp2pError> {
        let id = Idp2pId::from_bytes(&self.id)?;
        let microledger: Idp2pMicroledger = Idp2pMicroledger::decode(&*self.microledger)?;
        id.ensure(&microledger.inception)?;
        // Decode inception bytes of microledger
        let inception: Idp2pInception = Idp2pInception::decode(&*microledger.inception)?;
        // Init current state to handle events
        let mut state = IdentityState {
            id: self.id.clone(),
            event_id: self.id.clone(),
            m: inception.m,
            r: inception.r,
            n: inception.n,
            key_hashs: inception.key_hashs.clone(),
            sdt_proof: inception.sdt_proof.clone(),
        };
        for event in microledger.events {
            let event_id = Idp2pId::from_bytes(&event.id)?;
            event_id.ensure(&event.payload)?;
            let payload: Idp2pEventPayload = Idp2pEventPayload::decode(event.payload.as_slice())?;
            if payload.previous != state.event_id {
                return Err(Idp2pError::InvalidPreviousEventLog);
            }
            let event_kind = payload
                .event_kind
                .ok_or(Idp2pError::RequiredField("event_kind".to_string()))?;
            let sig_len = event.signatures.len();
            for s in event.signatures {
                if !state.key_hashs.contains(&s.hash) {
                    return Err(Idp2pError::Other);
                }
                let key_hash = Idp2pLedgerPublicHash::from_multi_bytes(&s.hash)?;
                key_hash.ensure_public(&s.key)?;

                let signer = Idp2pLedgerPublicKey::new(key_hash.code(), &s.key)?;
                signer.verify(&event.payload, &s.sig)?;
            }
            use EventKind::*;
            match event_kind {
                Mutation(sdt_proof) => {
                    if sig_len < state.m as usize {
                        return Err(Idp2pError::Other);
                    }
                    state.sdt_proof = sdt_proof;
                }
                Recover(recover) => {
                    if sig_len < state.r as usize {
                        return Err(Idp2pError::Other);
                    }
                    state.key_hashs = recover.key_hashs;
                    state.m = recover.m;
                    state.r = recover.r;
                    state.n = recover.n;
                    state.sdt_proof = recover.sdt_proof;
                }
            }
            state.event_id = event.id;
        }
        Ok(state)
    }
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
