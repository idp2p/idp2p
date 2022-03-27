use crate::raw::RawWallet;
use crate::secret::SecretWallet;
use idp2p_common::ed_secret::EdSecret;
use idp2p_common::{anyhow::Result, base64url, serde_json};
use idp2p_core::store::IdStore;
use idp2p_didcomm::jpm::Jpm;
use idp2p_didcomm::jwe::Jwe;
use idp2p_didcomm::jwm::Jwm;
use idp2p_didcomm::jws::Jws;
use std::sync::Arc;

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
    pub fn send_message(
        &mut self,
        id_store: Arc<IdStore>,
        to: &str,
        message: &str,
    ) -> Result<String> {
        let id_state = id_store.state.lock().unwrap();
        let to_did = id_state.entries.get(to).map(|entry| entry.clone()).unwrap();
        let jwm = Jwm::new(self.raw.identity.clone(), to_did.did, message);
        let enc_secret = EdSecret::from_bytes(&self.secret.keyagreement_secret.to_vec());
        let jwe = jwm.seal(enc_secret)?;
        let json = idp2p_common::serde_json::to_string(&jwe)?;
        println!("{json}");
        Ok(json)
    }

    pub fn handle_jwm(&mut self, id_store: Arc<IdStore>, jwm: &str) -> Result<()> {
        let doc = self.raw.identity.document.clone().unwrap();
        let jwe: Jwe = serde_json::from_str(jwm)?;
        if doc.get_first_keyagreement() != jwe.recipients[0].header.kid {
            idp2p_common::anyhow::bail!("INVALID_KID");
        }
        let dec_secret = EdSecret::from_bytes(&self.secret.keyagreement_secret);
        let json = jwe.decrypt(dec_secret)?;
        let jws: Jws = serde_json::from_str(&json)?;
        let jpm: Jpm = base64url::decode(&jws.payload)?;
        let id_state = id_store.state.lock().unwrap();
        let from = id_state
            .entries
            .get(&jpm.from)
            .map(|entry| entry.clone())
            .unwrap();
        jws.verify(from.did)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use idp2p_common::ed_secret::EdSecret;
    use idp2p_core::did::Identity;
    use idp2p_core::store::IdEntry;
    use idp2p_core::store::IdState;
    use idp2p_core::IdentityEvent;
    use std::collections::BTreeMap;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use tokio::sync::mpsc::channel;

    #[test]
    fn send_message_test() -> Result<()> {
        let (mut session, id_store) = init();
        session.send_message(id_store, "", "Heyyy")?;
        Ok(())
    }

    fn init() -> (WalletSession, Arc<IdStore>) {
        let secret = EdSecret::new();
        let did = Identity::from_secret(secret.clone());
        let id = did.id.clone();
        let (tx, mut rx) = channel::<IdentityEvent>(100);
        let entry = IdEntry::new(did.clone());
        let mut entries = HashMap::new();
        entries.insert(id, entry);
        let id_store = IdStore {
            state: Mutex::new(IdState {
                entries: entries,
                events: BTreeMap::new(),
            }),
            event_sender: tx.clone(),
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
    }
}
