use idp2p_common::chrono::Utc;
use idp2p_common::ed_secret::EdSecret;
use idp2p_common::encode_vec;
use idp2p_common::{anyhow::Result, serde_json};
use idp2p_core::did::identity::Identity;
use idp2p_core::didcomm::jwe::Jwe;
use idp2p_core::didcomm::jwm::{Jwm, JwmBody};
use idp2p_core::didcomm::jws::Jws;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct SecretWallet {
    #[serde(with = "encode_vec")]
    pub(crate) assertion_secret: Vec<u8>,
    #[serde(with = "encode_vec")]
    pub(crate) authentication_secret: Vec<u8>,
    #[serde(with = "encode_vec")]
    pub(crate) keyagreement_secret: Vec<u8>,
}

#[derive(PartialEq, Debug, Clone)]
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
    pub fn new(secret: SecretWallet, pwd: &str) -> Self {
        let created_at = Utc::now().timestamp();
        WalletSession {
            secret: secret,
            created_at: created_at,
            expire_at: created_at + 24 * 60 * 60,
            password: pwd.to_owned(),
        }
    }

    pub fn new_with_secret(secret: EdSecret, pwd: &str) -> Self {
        let wallet_secret = SecretWallet {
            assertion_secret: secret.to_bytes().to_vec(),
            authentication_secret: secret.to_bytes().to_vec(),
            keyagreement_secret: secret.to_bytes().to_vec(),
        };
        Self::new(wallet_secret, pwd)
    }

    pub fn create_jwm(&self, from: &str, to: Identity, body: JwmBody) -> Result<String> {
        let jwm = Jwm::new(from, &to.id, body);
        let enc_secret = EdSecret::from_bytes(&self.secret.keyagreement_secret.to_vec());
        let to_doc = to.to_document();
        let agreement_kid: String = to_doc.get_first_keyagreement();
        let agreement_method = to_doc.get_verification_method(&agreement_kid).unwrap();
        let to_public: [u8; 32] = agreement_method.bytes.try_into().unwrap();
        let jwe = jwm.seal(enc_secret, to_public)?;
        let json = idp2p_common::serde_json::to_string(&jwe)?;
        Ok(json)
    }

    pub fn resolve_jwe(&self, jwe_str: &str, agreement_key: &str) -> Result<Jws> {
        let jwe: Jwe = serde_json::from_str(jwe_str)?;
        if agreement_key != jwe.recipients[0].header.kid {
            eprintln!("{:?}", jwe.recipients[0].header.kid);
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
            expire_at: self.expire_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use idp2p_common::ed_secret::EdSecret;
    use idp2p_core::{did::identity::Identity, didcomm::jpm::Jpm};
    use std::str::FromStr;

    #[test]
    fn create_resolve_test() -> Result<()> {
        let (from_did, from_session) = init();
        let (to_did, to_session) = init();

        let jwe = from_session.create_jwm(
            &from_did.id,
            to_did.clone(),
            JwmBody::Message("Heeyy".to_owned()),
        )?;
        let agreement_kid = to_did.to_document().get_first_keyagreement();
        let jws = to_session.resolve_jwe(&jwe, &agreement_kid)?;
        let jpm = Jpm::from_str(&jws.payload);

        Ok(())
    }

    fn init() -> (Identity, WalletSession) {
        let secret = EdSecret::new();
        let did = Identity::from_secret(secret.clone());
        let secret_wallet = SecretWallet {
            assertion_secret: secret.to_bytes().to_vec(),
            authentication_secret: secret.to_bytes().to_vec(),
            keyagreement_secret: secret.to_bytes().to_vec(),
        };
        let session = WalletSession {
            secret: secret_wallet,
            created_at: 0,
            expire_at: 0,
            password: "password".to_owned(),
        };
        (did, session)
    }
}
