use crate::eventlog::{
    EventLog, EventLogChange,
};
use crate::IdentityError;
use crate::{generate_cid, hash, RecoveryKey, SignerKey};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct MicroLedgerState {
    pub current_event_id: String,
    pub current_signer_key: SignerKey,
    pub current_recovery_key: RecoveryKey,
    pub current_proofs: HashMap<Vec<u8>, Vec<u8>>, // extract only current value
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct MicroLedgerInception {
    pub signer_key: SignerKey,
    pub recovery_key: RecoveryKey,
}

impl MicroLedgerInception {
    pub fn get_id(&self) -> String {
        generate_cid(self)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct MicroLedger {
    pub id: String, // incepiton id
    pub inception: MicroLedgerInception,
    pub events: Vec<EventLog>,
}

impl MicroLedger {
    pub fn new(signer_key: Vec<u8>, recovery_key_digest: Vec<u8>) -> MicroLedger {
        let signer_key = SignerKey::new(signer_key);
        let recovery_key = RecoveryKey::new(recovery_key_digest);
        let inception = MicroLedgerInception {
            signer_key: signer_key,
            recovery_key: recovery_key,
        };
        let id = inception.get_id();
        MicroLedger {
            id,
            inception,
            events: vec![],
        }
    }

    pub fn verify(&self, cid: String) -> Result<MicroLedgerState, IdentityError> {
        let mut state = MicroLedgerState {
            current_event_id: self.inception.get_id(),
            current_signer_key: self.inception.signer_key.clone(),
            current_recovery_key: self.inception.recovery_key.clone(),
            current_proofs: HashMap::new(),
        };
        check!(cid == self.inception.get_id(), IdentityError::InvalidId);
        for event in &self.events {
            let previous_valid = event.payload.previous == state.current_event_id;
            check!(previous_valid, IdentityError::InvalidLedger);
            let event_valid = event.verify(state.current_signer_key.public.clone());
            check!(event_valid, IdentityError::InvalidLedger);
            match &event.payload.change {
                EventLogChange::PutProof(stmt) => {
                    let signer_valid = state.current_signer_key.public == event.payload.signer_key.clone();
                    check!(signer_valid, IdentityError::InvalidLedger);
                    state.current_proofs.insert(
                         stmt.key.clone(),
                         stmt.value.clone(),
                    );
                }
                EventLogChange::Recover(recovery) => {
                    let recovery_key_digest = hash(&event.payload.signer_key.clone());
                    let rec_valid = recovery_key_digest == state.current_recovery_key.digest;
                    check!(rec_valid, IdentityError::InvalidLedger);
                    state.current_signer_key = recovery.next_signer_key.clone();
                    state.current_recovery_key = recovery.next_recovery_key.clone();
                }
            }
            state.current_event_id = event.get_id();
        }
        Ok(state)
    }

    pub fn get_previous_id(&self) -> String {
        let previous_id = if self.events.len() == 0 {
            self.inception.get_id()
        } else {
            let e = self.events.last().unwrap();
            e.get_id()
        };
        previous_id
    }
}

#[cfg(test)]
mod tests {
    use crate::eventlog::ProofStatement;
use super::*;
    use crate::*;
    #[test]
    fn generate_test() {
        let (incepiton_secret, incepiton_public) = create_verification_key();
        let mut ledger = MicroLedger::new(incepiton_public.clone(), hash(&incepiton_public));
        let proof_stmt = ProofStatement {
            key: vec![0],
            value: vec![0],
        };
        /*pome.add_statement(incepiton_secret.clone(), proof_stmt.clone());
        pome.add_statement(incepiton_secret.clone(), proof_stmt.clone());
        let (new_secret, new_public) = create_keypair();
        let rec = RecoverStatement {
            next_signer_key: SignerKey::new(new_public.clone()),
            next_recovery_key: RecoveryKey::new(hash(&new_secret)),
        };
        pome.recover(incepiton_secret.clone(), rec);
        let mut new_pome = pome.clone();
        new_pome.add_statement(new_secret.clone(), proof_stmt.clone());
        //pome.add_statement(incepiton_secret.clone(), proof_stmt.clone());
        let r = pome.is_next_ledger(new_pome.clone());
        assert!(r.is_ok());*/
    }
}
