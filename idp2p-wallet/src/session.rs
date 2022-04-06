use idp2p_common::ed_secret::EdSecret;
use idp2p_common::{anyhow::Result, serde_json};
use idp2p_core::did::Identity;
use idp2p_core::did_doc::IdDocument;
use idp2p_didcomm::jwe::Jwe;
use idp2p_didcomm::jws::Jws;
use idp2p_didcomm::jwm::Jwm;
use serde::{Deserialize, Serialize};
use idp2p_common::encode_vec;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct SecretWallet {
    #[serde(with = "encode_vec")]
    pub(crate) assertion_secret: Vec<u8>,
    #[serde(with = "encode_vec")]
    pub(crate) authentication_secret: Vec<u8>,
    #[serde(with = "encode_vec")]
    pub(crate) keyagreement_secret: Vec<u8>,
}

#[derive( PartialEq, Debug, Clone)]
pub struct WalletSession {
    pub secret: SecretWallet,
    pub created_at: i64,
    pub expire_at: i64,
    pub password: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct SessionState {
    pub created_at: i64,
    pub expire_at: i64,
}

impl WalletSession {
    pub fn create(secret: EdSecret, pwd: &str) -> Self{
        let wallet_secret = SecretWallet{
            assertion_secret: secret.to_bytes().to_vec(),
            authentication_secret: secret.to_bytes().to_vec(),
            keyagreement_secret: secret.to_bytes().to_vec(),
        };
        Self::new(wallet_secret, pwd)
    }

    pub fn new(secret: SecretWallet, pwd: &str) -> Self{
        WalletSession {
            secret: secret,
            created_at: 0,
            expire_at: 0,
            password: pwd.to_owned(),
        }
    }
    pub fn create_jwm(&self, jwm: Jwm) -> Result<String> {
        let enc_secret = EdSecret::from_bytes(&self.secret.keyagreement_secret.to_vec());
        let jwe = jwm.seal(enc_secret)?;
        let json = idp2p_common::serde_json::to_string(&jwe)?;
        Ok(json)
    }

    pub fn resolve_jwe(&self, jwe_str: &str, doc: IdDocument) -> Result<Jws> {
        let jwe: Jwe = serde_json::from_str(jwe_str)?;
        if doc.get_first_keyagreement() != jwe.recipients[0].header.kid {
            idp2p_common::anyhow::bail!("INVALID_KID");
        }
        let dec_secret = EdSecret::from_bytes(&self.secret.keyagreement_secret);
        let json = jwe.decrypt(dec_secret)?;
        let jws: Jws = serde_json::from_str(&json)?;
        Ok(jws)
    }

    pub fn to_state(&self) -> SessionState {
        SessionState {
            created_at: self.created_at,
            expire_at: self.expire_at
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use idp2p_common::ed_secret::EdSecret;
    use idp2p_core::{did::Identity, IdProfile};
    use idp2p_didcomm::jpm::Jpm;
/* 
    #[test]
    fn create_resolve_test() -> Result<()> {
        let mut session_from = init();
        let mut session_to = init();

        let jwe = session_from.create_jwm(
            session_to.raw.identity.clone(),
            JwmBody::Message("Heeyy".to_owned()),
        )?;
        let jws = session_to.resolve_jwe(&jwe)?;
        let jpm = Jpm::from_str(&jws.payload);
        eprintln!("{:?}", jpm);
        Ok(())
    }

    fn init() -> WalletSession {
        let secret = EdSecret::new();
        let did = Identity::from_secret(secret.clone());
        let p = IdProfile::new("adem", &vec![]);
        let raw_wallet = RawWallet::new(p, did);
        let secret_wallet = SecretWallet {
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
        session
    }*/
}
