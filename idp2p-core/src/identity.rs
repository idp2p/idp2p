use crate::{
    error::Idp2pError,
    idp2p_proto::{self},
};
use idp2p_common::multi::{
    id::Idp2pId,
    ledgerkey::{Idp2pLedgerPublicDigest, Idp2pLedgerPublicKey},
};
use prost::Message;

#[derive(PartialEq, Debug, Clone)]
pub struct IdentityState {
    pub id: Vec<u8>,
    pub latest_event_id: Vec<u8>,
    pub owner_next_pk_hash: Vec<u8>,
    pub root_next_pk_hash: Vec<u8>,
    pub sdt_roots: Vec<Vec<u8>>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Identity {
    pub id: Vec<u8>,
    pub microledger: Vec<u8>,
}

impl Identity {
    pub fn new() -> Result<Identity, Idp2pError> {
        todo!()
    }

    pub fn mutate(&mut self, sdt_root: &[u8]) -> Result<(), Idp2pError> {
        let id = Idp2pId::from_bytes(&self.id)?;
        todo!()
    }

    /// Verify an identity and get state of identity
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
            sdt_roots: vec![],
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
                EventKind::Mutation(mutation) =>{
                    let owner_next_key_hash =
                        Idp2pLedgerPublicDigest::from_multi_bytes(&state.owner_next_pk_hash)?;
                        owner_next_key_hash.ensure_public(&mutation.owner_pk)?;
                    let signer =
                        Idp2pLedgerPublicKey::new(owner_next_key_hash.code(), &mutation.owner_pk)?;
                    signer.verify(&event.payload, &event.signature)?;
                    state.owner_next_pk_hash = mutation.owner_next_pk_hash;
                    state.sdt_roots.push(mutation.sdt_root);
                },
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
    }
}
