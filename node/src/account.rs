use crate::file_store::FileStore;
use crate::IdentityCommand;
use idp2p_core::create_secret_key;
use idp2p_core::did::Identity;
use idp2p_core::did_doc::CreateDocInput;
use idp2p_core::eventlog::ProofStatement;
use idp2p_core::eventlog::RecoverStatement;
use idp2p_core::hash;
use idp2p_core::to_verification_publickey;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Account {
    pub id: String,
    pub next_secret: Vec<u8>,
    pub recovery_secret: Vec<u8>,
    pub assertion_secret: Vec<u8>,
    pub authentication_secret: Vec<u8>,
    pub agreement_secret: Vec<u8>,
}

impl Account {
    pub fn new(id: &str, sec: &[u8], rec: &[u8]) -> Account {
        Account {
            id: id.to_owned(),
            next_secret: sec.to_owned(),
            recovery_secret: rec.to_owned(),
            assertion_secret: vec![],
            authentication_secret: vec![],
            agreement_secret: vec![],
        }
    }
}
pub fn handle_cmd(input: &str) -> Option<IdentityCommand> {
    let split = input.split(" ");
    let input: Vec<&str> = split.collect();
    match input[0] {
        "create" => {
            let name = input[1];
            let next_secret = create_secret_key();
            let recovery_secret = create_secret_key();
            let identity = Identity::new(&hash(&next_secret), &hash(&recovery_secret));
            let account = Account::new(&identity.id, &next_secret, &recovery_secret);
            FileStore.put("accounts", name, account);
            return Some(IdentityCommand::Post { did: identity });
        }
        "get" => {
            let id = input[1].to_string();
            return Some(IdentityCommand::Get { id });
        }
        "set-document" => {
            let name = input[1].to_string();
            if let Some(acc) = FileStore.get::<Account>("accounts", &name) {
                let next_secret = create_secret_key();
                let assertion_secret = create_secret_key();
                let authentication_secret = create_secret_key();
                let keyagreement_secret = create_secret_key();
                let mut identity = FileStore.get::<Identity>("identities", &acc.id).unwrap();
                let input = CreateDocInput {
                    id: identity.id.clone(),
                    assertion_key: to_verification_publickey(&assertion_secret),
                    authentication_key: to_verification_publickey(&authentication_secret),
                    keyagreement_key: to_verification_publickey(&keyagreement_secret),
                    service: vec![],
                };
                let key_digest = hash(&to_verification_publickey(&next_secret));
                identity.create_document(&acc.next_secret, &key_digest, input);
                FileStore.put("accounts", &name, acc);
                return Some(IdentityCommand::Post { did: identity });
            }
        }
        "set-proof" => {
            let name = input[1].to_string();
            let key = input[2].to_string();
            let value = input[3].to_string();
            if let Some(acc) = FileStore.get::<Account>("accounts", &name) {
                let mut identity = FileStore.get::<Identity>("identities", &acc.id).unwrap();
                let proof = ProofStatement {
                    key: key.as_bytes().to_vec(),
                    value: value.as_bytes().to_vec(),
                };
                let change = idp2p_core::eventlog::EventLogChange::SetProof(proof);
                let key_digest = hash(&to_verification_publickey(&acc.next_secret));
                identity.save_event(&acc.next_secret, &key_digest, change);
                return Some(IdentityCommand::Post { did: identity });
            }
        }
        "recover" => {
            let name = input[1].to_string();
            if let Some(acc) = FileStore.get::<Account>("accounts", &name) {
                let mut identity = FileStore.get::<Identity>("identities", &acc.id).unwrap();
                let stmt = RecoverStatement {
                    recovery_key_digest: hash(&acc.next_secret),
                };
                let change = idp2p_core::eventlog::EventLogChange::Recover(stmt);
                let key_digest = hash(&to_verification_publickey(&acc.next_secret));
                identity.save_event(&acc.next_secret, &key_digest, change);
                return Some(IdentityCommand::Post { did: identity });
            }
        }
        _ => {
            println!("Unknown command");
            return None;
        }
    }
    None
}
