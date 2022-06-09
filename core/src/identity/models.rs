use idp2p_common::multi::{
    agreement_key::Idp2pAgreementKey, key::Idp2pKey, key_digest::Idp2pKeyDigest,
    keypair::Idp2pKeypair,
};
use serde::{Serialize, Deserialize};
use idp2p_common::serde_vec::serde_vec;

// Can be used new identity or change
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(tag = "event_type")]
pub enum IdEvent {
    CreateAssertionKey {
        #[serde(with = "serde_vec")]
        id: Vec<u8>,
        key: Idp2pKey,
    },
    CreateAuthenticationKey {
        #[serde(with = "serde_vec")]
        id: Vec<u8>,
        key: Idp2pKey,
    },
    CreateAgreementKey {
        #[serde(with = "serde_vec")]
        id: Vec<u8>,
        key: Idp2pAgreementKey,
    },
    SetProof {
        #[serde(with = "serde_vec")]
        key: Vec<u8>,
        #[serde(with = "serde_vec")]
        value: Vec<u8>,
    },
    RevokeAssertionKey(#[serde(with = "serde_vec")] Vec<u8>),
    RevokeAuthenticationKey(#[serde(with = "serde_vec")] Vec<u8>),
    RevokeAgreementKey(#[serde(with = "serde_vec")] Vec<u8>),
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(tag = "type")]
pub enum ChangeType {
    AddEvents{ events: Vec<IdEvent>},
    Recover(Idp2pKeyDigest),
}

pub struct CreateIdentityInput {
    // Next key digest(multikey digest)
    pub next_key_digest: Idp2pKeyDigest,
    // Recovery key digest(multikey digest)
    pub recovery_key_digest: Idp2pKeyDigest,
    pub events: Vec<IdEvent>,
}

#[derive(Debug)]
pub struct ChangeInput {
    pub next_key_digest: Idp2pKeyDigest,
    pub signer_keypair: Idp2pKeypair,
    pub change: ChangeType,
}
