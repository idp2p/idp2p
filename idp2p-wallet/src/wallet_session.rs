use crate::raw_wallet::RawWallet;
use crate::secret_wallet::SecretWallet;
use idp2p_common::ed_secret::EdSecret;
use idp2p_common::{anyhow::Result, base64url, serde_json};
use idp2p_didcomm::jpm::Jpm;
use idp2p_didcomm::jwe::Jwe;
use idp2p_didcomm::jwm::Jwm;
use idp2p_didcomm::jws::Jws;
use idp2p_node::store::IdShared;

pub struct WalletSession {
    pub raw: RawWallet,
    pub secret: SecretWallet,
    pub created_at: i64,
    pub expire_at: i64,
    pub password: String,
    pub salt: [u8; 16],
    pub iv: [u8; 12],
    pub id_shared: IdShared
}

impl WalletSession {
    pub fn send_message(&mut self, to: &str, message: &str) -> Result<String> {
        let id_state = self.id_shared.state.lock().unwrap();
        let to_did = id_state.entries.get(to).map(|entry| entry.clone()).unwrap();
        let jwm = Jwm::new(self.raw.identity.clone(), to_did.did, message);
        let enc_secret = EdSecret::from_bytes(&self.secret.keyagreement_secret.to_vec());
        let jwe = jwm.seal(enc_secret)?;
        let json = idp2p_common::serde_json::to_string(&jwe)?;
        Ok(json)
    }

    pub fn handle_jwm(&mut self, jwm: &str) -> Result<()> {
        let doc = self.raw.identity.document.clone().unwrap();
        let jwe: Jwe = serde_json::from_str(jwm)?;
        if doc.get_first_keyagreement() != jwe.recipients[0].header.kid {
            idp2p_common::anyhow::bail!("INVALID_KID");
        }
        let dec_secret = EdSecret::from_bytes(&self.secret.keyagreement_secret);
        let json = jwe.decrypt(dec_secret)?;
        let jws: Jws = serde_json::from_str(&json)?;
        let jpm: Jpm = base64url::decode(&jws.payload)?;
        let id_state = self.id_shared.state.lock().unwrap();
        let from = id_state
            .entries
            .get(&jpm.from)
            .map(|entry| entry.clone())
            .unwrap();
        jws.verify(from.did)?;
        Ok(())
    }
}
