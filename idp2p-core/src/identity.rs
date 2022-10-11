use std::collections::HashMap;

use idp2p_common::multi::{
    id::{Idp2pCodec, Idp2pId},
    ledgerkey::{Idp2pLedgerKeypair, Idp2pLedgerPublicDigest, Idp2pLedgerPublicKey},
};

use crate::{codec::proto::id_decoder::ProtoIdentityDecoder, error::Idp2pError};

/// Identity event input
///
/// When a program wants to create or change identity, it uses the enum.
#[derive(PartialEq, Debug, Clone)]
pub enum IdEvent {
    CreateAssertionKey {
        id: Vec<u8>,
        multi_bytes: Vec<u8>,
    },
    CreateAuthenticationKey {
        id: Vec<u8>,
        multi_bytes: Vec<u8>,
    },
    CreateAgreementKey {
        id: Vec<u8>,
        multi_bytes: Vec<u8>,
    },
    /// `key`: proof key as bytes , `value`: proof value as bytes
    SetProof {
        key: Vec<u8>,
        value: Vec<u8>,
    },
    /// Id of assertion key
    RevokeAssertionKey(Vec<u8>),
    /// Id of authentication key
    RevokeAuthenticationKey(Vec<u8>),
    /// Id of agreement key
    RevokeAgreementKey(Vec<u8>),
}

#[derive(PartialEq, Debug, Clone)]
pub enum MutationKind {
    AddEvents { events: Vec<IdEvent> },
    Recover(Vec<u8>),
}

#[derive(PartialEq, Debug, Clone)]
pub struct CreateInput {
    pub timestamp: i64,
    // Next key digest(multikey digest)
    pub next_key_digest: Vec<u8>,
    // Recovery key digest(multikey digest)
    pub recovery_key_digest: Vec<u8>,
    pub events: Vec<IdEvent>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct MutateInput {
    pub next_key_digest: Vec<u8>,
    pub signer_keypair: Idp2pLedgerKeypair,
    pub mutation: MutationKind,
}

#[derive(PartialEq, Debug, Clone)]
pub struct AssertionPublicKeyState {
    pub id: Vec<u8>,
    pub valid_at: i64,
    pub expired_at: Option<i64>,
    pub key_bytes: Vec<u8>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct AuthenticationPublicKeyState {
    pub id: Vec<u8>,
    pub valid_at: i64,
    pub expired_at: Option<i64>,
    pub key_bytes: Vec<u8>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct AgreementPublicKeyState {
    pub id: Vec<u8>,
    pub valid_at: i64,
    pub expired_at: Option<i64>,
    pub key_bytes: Vec<u8>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct ProofState {
    pub valid_at: i64,
    pub expired_at: Option<i64>,
    pub value: Vec<u8>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct IdentityState {
    pub id: Vec<u8>,
    pub last_event_id: Vec<u8>,
    pub next_key_digest: Vec<u8>,
    pub recovery_key_digest: Vec<u8>,
    pub assertion_keys: Vec<AssertionPublicKeyState>,
    pub authentication_keys: Vec<AuthenticationPublicKeyState>,
    pub agreement_keys: Vec<AgreementPublicKeyState>,
    pub proofs: HashMap<Vec<u8>, ProofState>,
}

impl IdentityState {
    pub fn next_signer_key(&self, signer_pk: &[u8]) -> Result<Idp2pLedgerPublicKey, Idp2pError> {
        let key_digest = Idp2pLedgerPublicDigest::from_multi_bytes(&self.next_key_digest)?;
        key_digest.ensure_public(signer_pk)?;
        Ok(Idp2pLedgerPublicKey::new(key_digest.code(), signer_pk)?)
    }
    pub fn next_rec_key(&self, signer_pk: &[u8]) -> Result<Idp2pLedgerPublicKey, Idp2pError> {
        let key_digest = Idp2pLedgerPublicDigest::from_multi_bytes(&self.recovery_key_digest)?;
        key_digest.ensure_public(signer_pk)?;
        Ok(Idp2pLedgerPublicKey::new(key_digest.code(), signer_pk)?)
    }
    pub fn get_latest_auth_key(&self) -> Option<&AuthenticationPublicKeyState> {
        self.authentication_keys.last()
    }
    pub fn get_latest_agree_key(&self) -> Option<&AgreementPublicKeyState> {
        self.agreement_keys.last()
    }
    pub fn get_auth_key_by_id(&self, kid: &[u8]) -> Option<&AuthenticationPublicKeyState> {
        self.authentication_keys.iter().find(|pk| pk.id == kid)
    }
    pub fn get_agree_key_by_id(&self, kid: &[u8]) -> Option<&AgreementPublicKeyState> {
        self.agreement_keys.iter().find(|pk| pk.id == kid)
    }
}
pub trait IdentityStateEventHandler<T> {
    fn handle_event(&mut self, timestamp: i64, event: T) -> Result<(), Idp2pError>;
}

#[derive(PartialEq, Debug, Clone)]
pub struct Identity {
    // Bytes of inception id(idp2p multi id)
    pub id: Vec<u8>,
    // Microledger bytes(can be protobuf, json ... encoded)
    pub microledger: Vec<u8>,
}

pub trait IdentityDecoder {
    fn new(&self, input: CreateInput) -> Result<Identity, Idp2pError>;
    fn change(&self, did: &mut Identity, input: MutateInput) -> Result<bool, Idp2pError>;
    fn verify(&self, did: &Identity, prev: Option<&Identity>) -> Result<IdentityState, Idp2pError>;
}

impl Identity {
    pub fn new_protobuf(input: CreateInput) -> Result<Identity, Idp2pError> {
        ProtoIdentityDecoder.new(input)
    }

    pub fn change(&mut self, input: MutateInput) -> Result<bool, Idp2pError> {
        let id = Idp2pId::from_bytes(&self.id)?;
        match id.codec {
            Idp2pCodec::Protobuf => ProtoIdentityDecoder.change(self, input),
            Idp2pCodec::Json => todo!(),
        }
    }

    /// Verify an identity and get state of identity
    pub fn verify(&self, prev: Option<&Identity>) -> Result<IdentityState, Idp2pError> {
        let id = Idp2pId::from_bytes(&self.id)?;
        match id.codec {
            Idp2pCodec::Protobuf => ProtoIdentityDecoder.verify(self, prev),
            Idp2pCodec::Json => todo!(),
        }
    }
}
