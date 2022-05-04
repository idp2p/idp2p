use idp2p_common::{
    encode_vec,
    key::{Idp2pAgreementKey, Idp2pKey, Idp2pKeyDigest},
    serde_with::skip_serializing_none,
};

use crate::idp2p_proto::{self, identity_event::IdentityEventType, Identity};
use serde::{Deserialize, Serialize};

#[skip_serializing_none]
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct VerificationMethod {
    pub id: String,
    pub controller: String,
    #[serde(rename = "type")]
    pub typ: String,
    #[serde(with = "encode_vec", rename = "publicKeyMultibase")]
    pub bytes: Vec<u8>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct IdDocument {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    pub id: String,
    pub controller: String,
    #[serde(rename = "verificationMethod")]
    pub verification_method: Vec<VerificationMethod>,
    #[serde(rename = "assertionMethod")]
    pub assertion_method: Vec<String>,
    pub authentication: Vec<String>,
    #[serde(rename = "keyAgreement")]
    pub key_agreement: Vec<String>,
}

pub struct CreateIdentityInput {
    pub hash_alg: u8,
    pub next_key_digest: Idp2pKeyDigest,
    pub recovery_key_digest: Idp2pKeyDigest,
    pub assertion_key: Option<Idp2pKey>,
    pub authentication_key: Option<Idp2pKey>,
    pub agreement_key: Option<Idp2pAgreementKey>,
}

impl From<CreateIdentityInput> for Identity {
    fn from(input: CreateIdentityInput) -> Self {
        let mut events: Vec<IdentityEventType> = vec![];
        if let Some(assertion_key) = input.assertion_key {
            events.push(IdentityEventType::SetAssertionKey(idp2p_proto::Idp2pKey {
                alg: assertion_key.get_alg(),
                public: assertion_key.get_public(),
            }));
        }
        /*let inception = did_proto::MicroledgerInception{

        }*/
        todo!()
    }
}

impl From<idp2p_proto::Identity> for IdDocument {
    fn from(identity: idp2p_proto::Identity) -> Self {
        todo!()
    }
}

pub fn verify(identity: idp2p_proto::Identity) {}
