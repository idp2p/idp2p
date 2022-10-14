use idp2p_common::multi::ledgerkey::Idp2pLedgerKeypair;


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
