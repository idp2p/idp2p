use idp2p_common::{
    anyhow::Result,
    ed25519_dalek::{PublicKey, Signature, Verifier},
    ed_secret::EdSecret,
    encode_vec,
};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Clone, Deserialize, Serialize)]
pub enum Idp2pRecoveryType {
    InLedger,
}

#[derive(PartialEq, Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Idp2pPublicKeyDigest {
    Idp2pEd25519 {
        #[serde(with = "encode_vec")]
        digest: Vec<u8>,
    },
}

#[derive(PartialEq, Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Idp2pPublicKey {
    Idp2pEd25519 {
        #[serde(with = "encode_vec")]
        public: Vec<u8>,
    },
}

#[derive(PartialEq, Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Idp2pAgreementPublicKey {
    Idp2pX25519 {
        #[serde(with = "encode_vec")]
        public: Vec<u8>,
    },
}

#[derive(PartialEq, Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum MicroledgerEvent {
    SetProof {
        #[serde(with = "encode_vec")]
        key: Vec<u8>,
        #[serde(with = "encode_vec")]
        value: Vec<u8>,
    },
    SetAssertionKey {
        key: Idp2pPublicKey,
    },
    SetAuthenticationKey {
        key: Idp2pPublicKey,
    },
    SetAgreementKey {
        key: Idp2pAgreementPublicKey,
    },
}

impl From<Idp2pPublicKeyDigest> for Vec<u8> {
    fn from(_: Idp2pPublicKeyDigest) -> Self {
        todo!()
    }
}

impl TryFrom<EdSecret> for Idp2pPublicKeyDigest {
    type Error = idp2p_common::anyhow::Error;

    fn try_from(secret: EdSecret) -> Result<Self, Self::Error> {
        Ok(Idp2pPublicKeyDigest::Idp2pEd25519 {
            digest: secret.to_publickey_digest()?.to_vec(),
        })
    }
}

impl From<EdSecret> for Idp2pPublicKey {
    fn from(secret: EdSecret) -> Self {
        Idp2pPublicKey::Idp2pEd25519 {
            public: secret.to_publickey().to_vec(),
        }
    }
}

impl From<EdSecret> for Idp2pAgreementPublicKey {
    fn from(secret: EdSecret) -> Self {
        Idp2pAgreementPublicKey::Idp2pX25519 {
            public: secret.to_key_agreement().to_vec(),
        }
    }
}

impl Idp2pPublicKey {
    pub fn verify(&self, data: &[u8], sig: &[u8]) -> Result<bool> {
        match &self {
            Self::Idp2pEd25519 { public } => {
                let public_key: PublicKey = PublicKey::from_bytes(public).unwrap();
                let signature_bytes: [u8; 64] = sig.try_into().unwrap();
                let signature = Signature::from(signature_bytes);
                Ok(public_key.verify(data, &signature).is_ok())
            }
        }
    }
}
/*impl From<did_proto::MicroledgerChange> for MicroledgerEvent {
    fn from(change: did_proto::MicroledgerChange) -> Self {
        match change.change.unwrap() {
            did_proto::microledger_change::Change::SetProof(set_proof) => {
                MicroledgerEvent::SetProof {
                    key: set_proof.key,
                    value: set_proof.value,
                }
            }
            did_proto::microledger_change::Change::SetAssertionKey(set_assertion_key) => {
                MicroledgerEvent::SetAssertionKey(Idp2pPublicKey {
                    r#type: set_assertion_key.key_type.into(),
                    public: set_assertion_key.public_key,
                })
            }
            did_proto::microledger_change::Change::SetAuthenticationKey(set_authentication_key) => {
                MicroledgerEvent::SetAuthenticationKey(Idp2pPublicKey {
                    r#type: set_authentication_key.key_type.into(),
                    public: set_authentication_key.public_key,
                })
            }
            did_proto::microledger_change::Change::SetAgreementKey(set_agreement_key) => {
                MicroledgerEvent::SetAgreementKey(Idp2pAgreementPublicKey {
                    r#type: set_agreement_key.key_type.into(),
                    public: set_agreement_key.public_key,
                })
            }
            _ => panic!(""),
        }
    }
}*/

pub mod event_log;
pub mod identity;
pub mod microledger;
