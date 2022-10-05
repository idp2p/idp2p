use std::{collections::HashMap, sync::Mutex};

use idp2p_common::multi::{
    agreement::Idp2pAgreementKeypair,
    authentication::{Idp2pAuthenticationKeypair, Idp2pAuthenticationPublicKey},
    id::Idp2pCodec,
    message::Idp2pMessage,
};
use tokio::sync::mpsc::Sender;

use crate::{
    error::Idp2pError,
    id_state::IdentityState,
    identity::Identity,
};

pub enum IdStoreEvent {
    ReceivedMessage(Vec<u8>),
}

pub enum IdStoreCommand {
    SendMessage { id: Vec<u8>, message: IdMessageBody },
}

#[derive(Debug)]
pub enum IdStoreOutEvent {
    MessageReceived(IdMessage),
}

#[derive(Debug)]
pub enum IdStoreOutCommand {
    PublishMessage { topic: String, message: Vec<u8> },
}

#[derive(PartialEq, Debug, Clone)]
pub struct IdEntry {
    // Identity ledger
    pub(crate) did: Identity,
    // State of requiring publish
    pub(crate) waiting_publish: bool,
    // Current state
    pub(crate) id_state: IdentityState,
}

pub struct IdDb {
    pub(crate) owner: IdentityState,
    pub(crate) auth_keypair: Idp2pAuthenticationKeypair,
    pub(crate) agree_keypair: Idp2pAgreementKeypair,
    pub(crate) identities: HashMap<Vec<u8>, IdEntry>,
}

pub struct IdStoreOptions {
    pub(crate) owner: Identity,
    pub(crate) auth_keypair: Idp2pAuthenticationKeypair,
    pub(crate) agree_keypair: Idp2pAgreementKeypair,
    pub(crate) event_sender: Sender<IdStoreOutEvent>,
    pub(crate) command_sender: Sender<IdStoreOutCommand>,
}

pub struct IdStore {
    pub(crate) db: Mutex<IdDb>,
    pub(crate) event_sender: Sender<IdStoreOutEvent>,
    pub(crate) command_sender: Sender<IdStoreOutCommand>,
    pub(crate) codec: Idp2pCodec,
}

impl IdDb {
    fn push_entry(&mut self, did: Identity) -> Result<(), Idp2pError> {
        let id = did.id.clone();
        let id_state = did.verify(None)?;
        let entry = IdEntry {
            waiting_publish: false,
            did: did,
            id_state: id_state,
        };
        self.identities.insert(id, entry);
        Ok(())
    }

    pub fn to_msg_handler_state(&self) -> MessageHandlerState {
        MessageHandlerState {
            agree_keypair: self.agree_keypair.clone(),
            auth_keypair: self.auth_keypair.clone(),
            from: self.owner.clone(),
        }
    }
}

impl IdStore {
    pub fn new(options: IdStoreOptions) -> Result<Self, Idp2pError> {
        let owner_state = options.owner.verify(None)?;
        let db = IdDb {
            owner: owner_state,
            auth_keypair: options.auth_keypair,
            agree_keypair: options.agree_keypair,
            identities: HashMap::new(),
        };
        let store = Self {
            db: Mutex::new(db),
            event_sender: options.event_sender,
            command_sender: options.command_sender,
            codec: Idp2pCodec::Protobuf,
        };
        Ok(store)
    }

    pub async fn handle_command(&self, cmd: IdStoreCommand) -> Result<(), Idp2pError> {
        let db = self.db.lock().unwrap();
        match cmd {
            IdStoreCommand::Get(id) => {
                let topic = String::from_utf8_lossy(&id).to_string();
                let event = IdStoreOutCommand::PublishGet(topic);
                self.command_sender.send(event).await?;
            }
            IdStoreCommand::SendMessage { id, message } => {
                let topic = String::from_utf8_lossy(&id).to_string();
                let to = db.identities.get(&id).expect("").id_state.clone();
                let encoded_msg = match self.codec {
                    Idp2pCodec::Protobuf => {
                        ProtoMessageHandler(db.to_msg_handler_state()).seal(to, message)?
                    }
                    Idp2pCodec::Json => todo!(),
                };
                let event = IdStoreOutCommand::PublishMessage {
                    topic: topic,
                    message: encoded_msg.message,
                };
                self.command_sender.send(event).await?;
            }
        }
        Ok(())
    }

    pub async fn handle_event(&self, topic: &str, event: IdStoreEvent) -> Result<(), Idp2pError> {
        let mut db = self.db.lock().unwrap();
        match event {
           IdStoreEvent::ReceivedMessage(msg) => {
                let entry = db.identities.get_mut(topic.as_bytes());
                if entry.is_some() {
                    let msg = Idp2pMessage::from_multi_bytes(&msg)?;
                    let dec_result = match msg.codec {
                        Idp2pCodec::Protobuf => {
                            ProtoMessageHandler(db.to_msg_handler_state()).decode(&msg.body)?
                        }
                        Idp2pCodec::Json => todo!(),
                    };

                    let from = db
                        .identities
                        .get(&dec_result.message.from)
                        .ok_or(Idp2pError::RequiredField("From".to_string()))?;
                    let signer_pk_state = from
                        .id_state
                        .get_auth_key_by_id(&dec_result.message.signer_kid)
                        .ok_or(Idp2pError::Other)?;
                    let signer_pk =
                        Idp2pAuthenticationPublicKey::from_multi_bytes(&signer_pk_state.key_bytes)?;
                    signer_pk.verify(&dec_result.agreement_data, &dec_result.message.proof)?;
                    let event = IdStoreOutEvent::MessageReceived(dec_result.message);
                    self.event_sender.send(event).await?;
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
            agreement::x25519::X25519Keypair, ledgerkey::Idp2pLedgerKeypair,
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
