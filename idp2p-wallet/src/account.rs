use anyhow::Result;
use anyhow::*;
use hmac::{Hmac, Mac, NewMac};
use idp2p_common::encode_vec;
use idp2p_common::secret::IdSecret;
use idp2p_core::did::Identity;
use idp2p_core::did_doc::CreateDocInput;
use idp2p_core::did_doc::IdDocument;
use serde::{Deserialize, Serialize};
use sha2::Sha512;

type HmacSha512 = Hmac<Sha512>;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Account {
    pub name: String,
    pub identity: Identity,
    #[serde(with = "encode_vec")]
    pub authentication_secret: Vec<u8>,
    #[serde(with = "encode_vec")]
    pub keyagreement_secret: Vec<u8>,
}

impl Account {
    pub fn create(name: &str) -> Result<Account> {
        let seed = idp2p_common::create_random::<16>();
        let mut mac = HmacSha512::new_varkey("idp2p seed".as_ref()).unwrap();
        mac.update(&seed);
        let bytes = mac.finalize().into_bytes().to_vec();
        let secret = bytes[..32].to_vec();
        let chain_code = bytes[32..64].to_vec();
        //let secret_key = SecretKey::from_bytes(&bytes[..32])?;
        //let chain_code = idp2p_common::create_random::<32>();
        let master_xpriv = "";
        let master_xpub = "";
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
        };
        let identity_doc = IdDocument::new(create_doc_input);
        /*let change = identity.save_document(identity_doc);
        let payload = identity.microledger.create_event(&signer_key, &next_key_digest, change);
        let proof = next_secret.sign(&payload);
        identity.microledger.save_event(payload, &proof);
        let store = FileStore {};
        let account = Account {
            name: name.to_owned(),
            identity: identity.clone(),
            next_secret: next_secret.to_bytes(),
            authentication_secret: next_secret.to_bytes(),
            keyagreement_secret: next_secret.to_bytes(),
        };
        println!("Created identity: {:?}", identity.id.clone());
        store.put("identities", &identity.id.clone(), identity);
        store.put("accounts", name, account);*/
        Err(anyhow!(""))
    }
}
