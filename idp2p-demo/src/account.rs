use idp2p_common::encode_vec;
use idp2p_common::secret::IdSecret;
use idp2p_common::store::FileStore;
use idp2p_common::store::IdStore;
use idp2p_core::did::Identity;
use idp2p_core::did_doc::CreateDocInput;
use idp2p_core::did_doc::IdDocument;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Account {
    pub name: String,
    pub id: String,
    #[serde(with = "encode_vec")]
    pub next_secret: Vec<u8>,
    #[serde(with = "encode_vec")]
    pub authentication_secret: Vec<u8>,
    #[serde(with = "encode_vec")]
    pub keyagreement_secret: Vec<u8>,
}

impl Account {
    pub fn init(name: &str) -> u16 {
        let base_path = format!("../target/{}", name);
        std::env::set_var("BASE_PATH", base_path.clone());
        let id_path = format!("{}/identities", base_path);
        std::fs::create_dir_all(id_path).unwrap();
        let acc_path = format!("{}/accounts", base_path);
        std::fs::create_dir_all(acc_path).unwrap();
        let mut port = 5000;
        if name == "bob" {
            port = 6000;
        }
        let next_secret = IdSecret::new();
        let signer_key = next_secret.to_verification_publickey();
        let next_key_digest = next_secret.to_publickey_digest();
        let recovery_key_digest = next_secret.to_publickey_digest();
        let mut identity = Identity::new(&next_key_digest, &recovery_key_digest);
        let create_doc_input = CreateDocInput {
            id: identity.id.clone(),
            assertion_key: next_secret.to_verification_publickey(),
            authentication_key: next_secret.to_verification_publickey(),
            keyagreement_key: next_secret.to_key_agreement_publickey(),
            service: vec![],
        };
        let identity_doc = IdDocument::new(create_doc_input);
        let change = identity.save_document(identity_doc);
        let payload = identity.microledger.create_event(&signer_key, &next_key_digest, change);
        let proof = next_secret.sign(&payload);
        identity.microledger.save_event(payload, &proof);
        let store = FileStore {};
        let account = Account {
            name: name.to_owned(),
            id: identity.id.clone(),
            next_secret: next_secret.to_bytes(),
            authentication_secret: next_secret.to_bytes(),
            keyagreement_secret: next_secret.to_bytes(),
        };
        println!("Created identity: {:?}", identity.id.clone());
        store.put("identities", &identity.id.clone(), identity);
        store.put("accounts", name, account);
        return port;
    }

    pub fn handle_command(&self, input: &str) {
        let split = input.split(" ");
        let input: Vec<&str> = split.collect();
        match input[0] {
            "get" => {
                let id = input[1].to_string();
                //return Some(IdentityCommand::Get { id });
            }
            _ => {
                println!("Unknown command");
            }
        }
    }
}
