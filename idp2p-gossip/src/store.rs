use std::{collections::HashMap, sync::Mutex};

use tokio::sync::mpsc::Sender;

use crate::{error::Idp2pError, id_state::IdentityState, identity::Identity};

#[derive(PartialEq, Debug, Clone)]
pub struct IdEntry {
    // Identity ledger
    did: Identity,
    // State of requiring publish
    waiting_publish: bool,
    // Current state
    id_state: IdentityState,
    // Subscribers
    messages: Vec<PeerId>
}

pub struct GossipState {
    tokens: HashSet<String>,
    identities: HashMap<Vec<u8>, IdEntry>,
}

pub struct GossipStore{
    state: Mutex<GossipState>
}

impl IdStore {
    fn push_entry(&mut self, did: Identity) -> Result<(), Idp2pError> {
        let id = did.id.clone();
        let id_state = did.verify(None)?;
        let entry = IdEntry {
            waiting_publish: false,
            did: did,
            id_state: id_state,
        };
        self.db.insert(id, entry);
        Ok(())
    }
}

impl IdStore {
    pub fn new() -> Result<Self, Idp2pError> {
        let store = Self {
            db: Mutex::new(HashMap::new())
        };
        Ok(store)
    }

    pub async fn handle_event(&self, topic: &str, event: IdStoreEvent) -> Result<(), Idp2pError> {
        let mut db = self.db.lock().unwrap();
        match event {
            IdStoreEvent::ReceivedGet => {
                let entry = db.identities.get_mut(topic.as_bytes());
                if let Some(entry) = entry {
                    if &entry.did.id == topic.as_bytes() {
                        log::info!("Published id: {:?}", topic);
                        let cmd = IdStoreOutEvent::PostPublished {
                            topic: topic.to_string(),
                            microledger: entry.did.microledger.clone(),
                        };
                        self.event_sender.send(cmd).await?;
                    } else {
                        entry.waiting_publish = true;
                        let cmd = IdStoreOutEvent::WaitAndPostPublished(topic.as_bytes().to_vec());
                        self.event_sender.send(cmd).await?;
                    }
                }
            }
            IdStoreEvent::ReceivedPost { last_event_id, did } => {
                let current = db.identities.get_mut(topic.as_bytes());
                let id = did.id.clone();
                log::info!("Got id: {:?}", id);
                match current {
                    // When identity is new
                    None => {
                        db.push_entry(did)?;
                        let event = IdStoreOutEvent::IdentityCreated(topic.as_bytes().to_vec());
                        self.event_sender.send(event).await?;
                    }
                    // There is a current identity
                    Some(entry) => {
                        // If there is a waiting publish, remove it
                        entry.waiting_publish = false;
                        // Identity has a new state
                        if last_event_id != entry.id_state.last_event_id {
                            //entry.did.is_next(did.clone())?;
                            entry.did = did.clone();
                            log::info!("Updated id: {:?}", did.id);
                            let event = IdStoreOutEvent::IdentityUpdated(topic.as_bytes().to_vec());
                            self.event_sender.send(event).await?;
                        } else {
                            log::info!("Skipped id: {:?}", did.id);
                            let event = IdStoreOutEvent::IdentitySkipped(topic.as_bytes().to_vec());
                            self.event_sender.send(event).await?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use idp2p_common::{
        chrono::Utc,
        multi::{
            agreement::{x25519::X25519Keypair, Idp2pAgreementKeypair},
            authentication::Idp2pAuthenticationKeypair,
            ledgerkey::Idp2pLedgerKeypair,
            verification::ed25519::Ed25519Keypair,
        },
    };

    use crate::identity::{CreateIdentityInput, IdEvent};

    use super::*;
    #[test]
    fn get_test() -> Result<(), Idp2pError> {
        let alice_ledger_keypair = Idp2pLedgerKeypair::Ed25519(Ed25519Keypair::generate());
        let bob_ledger_keypair = Idp2pLedgerKeypair::Ed25519(Ed25519Keypair::generate());
        let alice_auth_keypair = Idp2pAuthenticationKeypair::Ed25519(Ed25519Keypair::generate());
        let bob_auth_keypair = Idp2pAuthenticationKeypair::Ed25519(Ed25519Keypair::generate());
        let alice_agree_keypair = Idp2pAgreementKeypair::X25519(X25519Keypair::generate());
        let bob_agree_keypair = Idp2pAgreementKeypair::X25519(X25519Keypair::generate());
        let alice_auth_pk = alice_auth_keypair.to_public_key();
        let alice_agree_pk = alice_agree_keypair.to_public_key();
        let bob_auth_pk = bob_auth_keypair.to_public_key();
        let bob_agree_pk = bob_agree_keypair.to_public_key();
        let alice_auth_event = IdEvent::CreateAuthenticationKey {
            id: alice_auth_pk.generate_id().to_vec(),
            multi_bytes: alice_auth_pk.to_multi_bytes(),
        };
        let alice_agree_event = IdEvent::CreateAgreementKey {
            id: alice_agree_pk.generate_id().to_vec(),
            multi_bytes: alice_agree_pk.to_multi_bytes(),
        };
        let bob_auth_event = IdEvent::CreateAuthenticationKey {
            id: bob_auth_pk.generate_id().to_vec(),
            multi_bytes: bob_auth_pk.to_multi_bytes(),
        };
        let bob_agree_event = IdEvent::CreateAgreementKey {
            id: bob_agree_pk.generate_id().to_vec(),
            multi_bytes: bob_agree_pk.to_multi_bytes(),
        };
        let alice_input = CreateIdentityInput {
            timestamp: Utc::now().timestamp(),
            next_key_digest: alice_ledger_keypair
                .to_public_key()
                .to_digest()?
                .to_multi_bytes(),
            recovery_key_digest: alice_ledger_keypair
                .to_public_key()
                .to_digest()?
                .to_multi_bytes(),
            events: vec![alice_auth_event, alice_agree_event],
        };
        let bob_input = CreateIdentityInput {
            timestamp: Utc::now().timestamp(),
            next_key_digest: bob_ledger_keypair
                .to_public_key()
                .to_digest()?
                .to_multi_bytes(),
            recovery_key_digest: bob_ledger_keypair
                .to_public_key()
                .to_digest()?
                .to_multi_bytes(),
            events: vec![bob_auth_event, bob_agree_event],
        };
        let alice_did = Identity::new_protobuf(alice_input)?;
        let bob_did = Identity::new_protobuf(bob_input)?;
        Ok(())
    }
}
