use std::collections::HashMap;

use idp2p_common::{key::{Idp2pKey,Idp2pAgreementKey}, digest::Idp2pKeyDigest, secret::Idp2pSecret};

#[derive(PartialEq, Debug, Clone)]
pub struct IdEvents {
    pub assertion_key: Option<Idp2pKey>,
    pub authentication_key: Option<Idp2pKey>,
    pub agreement_key: Option<Idp2pAgreementKey>,
    pub proofs: HashMap<Vec<u8>, Vec<u8>>,
}

pub struct CreateIdentityInput {
    pub next_key_digest: Idp2pKeyDigest,
    pub recovery_key_digest: Idp2pKeyDigest,
    pub events: IdEvents,
}

pub struct ChangeInput {
    pub next_key_digest: Idp2pKeyDigest,
    pub recovery_key_digest: Idp2pKeyDigest,
    pub signer: Idp2pSecret,
    pub events: IdEvents,
}

pub struct RecoverInput {
    pub next_key_digest: Idp2pKeyDigest,
    pub recovery_key_digest: Idp2pKeyDigest,
    pub signer: Idp2pSecret,
}
