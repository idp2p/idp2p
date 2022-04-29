use std::collections::HashMap;

use crate::IdentityError;

use super::{
    event_log::EventLog, Idp2pAgreementPublicKey, Idp2pPublicKey, Idp2pPublicKeyDigest,
    Idp2pRecoveryType, MicroledgerEvent,
};
use idp2p_common::{anyhow::Result, ed_secret::EdSecret, generate_cid, Idp2pCodec};
use serde::{Deserialize, Serialize};

/// This is the inception of identity
#[derive(PartialEq, Debug, Clone, Deserialize, Serialize)]
pub struct MicroledgerInception {
    // Recovery type, can be blockchain or any database
    pub recovery_type: Idp2pRecoveryType,
    // Next signer key digest to sign microledger events.
    // It is recommended to change every transaction
    pub next_key_digest: Idp2pPublicKeyDigest,
    // Recovery signer key digest
    pub recovery_key_digest: Idp2pPublicKeyDigest,
    // List of microledger events
    pub events: Vec<MicroledgerEvent>,
}

#[derive(PartialEq, Debug, Clone, Deserialize, Serialize)]
pub struct Microledger {
    // Inception of microledger
    pub inception: MicroledgerInception,
    // List of event logs
    pub event_logs: Vec<EventLog>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct MicroledgerState {
    pub event_log_id: Vec<u8>,
    pub next_key_digest: Idp2pPublicKeyDigest,
    pub recovery_key_digest: Idp2pPublicKeyDigest,
    pub assertion_keys: Vec<Idp2pPublicKey>,
    pub authentication_key: Option<Idp2pPublicKey>,
    pub agreement_key: Option<Idp2pAgreementPublicKey>,
    pub ledger_proofs: HashMap<Vec<u8>, Vec<u8>>,
}

impl MicroledgerInception {
    pub fn get_id(&self) -> String {
        generate_cid(self, Idp2pCodec::MsgPack).unwrap()
    }
}

impl Microledger {
    pub fn verify(&self, cid: &str) -> Result<MicroledgerState, IdentityError> {
        check!(cid == self.inception.get_id(), IdentityError::InvalidId);
        for event_log in &self.event_logs {}
        Err(IdentityError::InvalidId)
    }
}

impl TryFrom<EdSecret> for Microledger {
    type Error = idp2p_common::anyhow::Error;

    fn try_from(secret: EdSecret) -> Result<Self, Self::Error> {
        let assertion_event = MicroledgerEvent::SetAssertionKey{key: secret.clone().into()};
        let authentication_event = MicroledgerEvent::SetAuthenticationKey{key: secret.clone().into()};
        let agreement_event = MicroledgerEvent::SetAgreementKey{key: secret.clone().into()};
        let inception = MicroledgerInception {
            recovery_type: Idp2pRecoveryType::InLedger,
            next_key_digest: secret.clone().try_into()?,
            recovery_key_digest: secret.clone().try_into()?,
            events: vec![assertion_event, authentication_event, agreement_event],
        };
        let ledger = Microledger {
            inception: inception,
            event_logs: vec![],
        };
        Ok(ledger)
    }
}
