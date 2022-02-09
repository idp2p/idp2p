use idp2p_common::encode_vec;
use idp2p_core::ver_cred::VerifiableCredential;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct WalletAccount {
    pub name: String,
    pub id: String,
    pub credentials: Vec<VerifiableCredential>,
    pub next_derivation_index: u32,
    pub recovery_derivation_index: u32,
    pub assertion_derivation_index: Option<u32>,
    pub authentication_derivation_index: Option<u32>,
    pub keyagreement_derivation_index: Option<u32>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct WalletAccountSecret {
    #[serde(with = "encode_vec")]
    pub next_secret: Vec<u8>,
    #[serde(with = "encode_vec")]
    pub recovery_secret: Vec<u8>,
    #[serde(with = "encode_vec")]
    pub assertion_secret: Vec<u8>,
    #[serde(with = "encode_vec")]
    pub authentication_secret: Vec<u8>,
    #[serde(with = "encode_vec")]
    pub keyagreement_secret: Vec<u8>,
}

/*impl WalletAccount {
    pub fn new(name: &str, seed: [u8;16], index: u32) -> Result<Self> {
        let next_key_digest = IdSecret::from(&next_secret).to_publickey_digest();
        let recovery_key_digest = IdSecret::from(&recovery_secret).to_publickey_digest();
        let mut identity = Identity::new(&next_key_digest, &recovery_key_digest);
        let create_doc_input = CreateDocInput {
            id: identity.id.clone(),
            assertion_key: IdSecret::from(&secrets.assertion_secret).to_verification_publickey(),
            authentication_key: IdSecret::from(&secrets.authentication_secret)
                .to_verification_publickey(),
            keyagreement_key: IdSecret::from(&secrets.keyagreement_secret)
                .to_key_agreement_publickey(),
        };
        let identity_doc = IdDocument::new(create_doc_input);
        let change = identity.save_document(identity_doc);
        let payload = identity.microledger.create_event(&signer_key, &next_key_digest, change);
        let proof = next_secret.sign(&payload);
        identity.microledger.save_event(payload, &proof);
        
        Err(anyhow!(""))
    }
}
*/