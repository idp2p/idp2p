use idp2p_core::did::Identity;
use idp2p_didcomm::jwm::JwmHandler;
use crate::raw::RawWallet;
use crate::secret::SecretWallet;
use idp2p_common::ed_secret::EdSecret;
use idp2p_common::{anyhow::Result, serde_json};
use idp2p_didcomm::jwe::Jwe;
use idp2p_didcomm::jwm::JwmBody;
use idp2p_didcomm::jws::Jws;

pub struct WalletSession {
    pub raw: RawWallet,
    pub secret: SecretWallet,
    pub created_at: i64,
    pub expire_at: i64,
    pub password: String,
    pub salt: [u8; 16],
    pub iv: [u8; 12],
}

impl WalletSession {
    pub fn create_jwm(&mut self, to: Identity, jwm: JwmBody) -> Result<String> {
        let jwm = self.raw.identity.new_jwm(to, jwm);
        let enc_secret = EdSecret::from_bytes(&self.secret.keyagreement_secret.to_vec());
        let jwe = jwm.seal(enc_secret)?;
        let json = idp2p_common::serde_json::to_string(&jwe)?;
        Ok(json)
    }

    pub fn resolve_jwe(&mut self, jwe_str: &str) -> Result<Jws> {
        let doc = self.raw.identity.document.clone().unwrap();
        let jwe: Jwe = serde_json::from_str(jwe_str)?;
        if doc.get_first_keyagreement() != jwe.recipients[0].header.kid {
            idp2p_common::anyhow::bail!("INVALID_KID");
        }
        let dec_secret = EdSecret::from_bytes(&self.secret.keyagreement_secret);
        let json = jwe.decrypt(dec_secret)?;
        let jws: Jws = serde_json::from_str(&json)?;
        Ok(jws)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use idp2p_common::ed_secret::EdSecret;
    use idp2p_core::did::Identity;
    use idp2p_core::store::IdEntry;
    use idp2p_core::store::IdState;
    use std::collections::HashMap;
    use std::sync::Mutex;

    /*#[test]
    fn send_message_test() -> Result<()> {
        let (mut session, id_store) = init();
        session.create_jwm(id_store, "", JwmBody::Message("Heeyy".to_owned()))?;
        Ok(())
    }

    fn init() -> (WalletSession) {
        let secret = EdSecret::new();
        let did = Identity::from_secret(secret.clone());
        let id = did.id.clone();
        let entry = IdEntry::new(did.clone());
        let mut entries = HashMap::new();
        entries.insert(id, entry);
        let id_store = IdStore {
            state: Mutex::new(IdState::new(entries)),
        };
        let id_store = Arc::new(id_store);
        let raw_wallet = RawWallet::new("adem", did);
        let secret_wallet = SecretWallet {
            next_index: 0,
            next_secret_index: 0,
            recovery_secret_index: 0,
            assertion_secret: secret.to_bytes().to_vec(),
            authentication_secret: secret.to_bytes().to_vec(),
            keyagreement_secret: secret.to_bytes().to_vec(),
        };
        let session = WalletSession {
            raw: raw_wallet,
            secret: secret_wallet,
            created_at: 0,
            expire_at: 0,
            password: "password".to_owned(),
            salt: [0u8; 16],
            iv: [0u8; 12],
        };
        (session, id_store)
    }*/
}
