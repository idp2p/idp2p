use crate::jpm::Jpm;
use crate::jws::Jws;
use idp2p_common::ed_secret::EdSecret;
use crate::jwe::Jwe;
use idp2p_core::did::Identity;
use serde::{Deserialize, Serialize};
use idp2p_common::{serde_json, chrono::Utc, anyhow::Result};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Jwm {
    pub id: String,
    pub from: Identity,
    pub to: Identity,
    pub created_time: i64,
    pub body: serde_json::Value,
}

impl Jwm {
    pub fn new(from: Identity, to: Identity, body: &str) -> Jwm {
        let id = idp2p_common::encode(&idp2p_common::create_random::<32>());
        Jwm {
            id: id,
            from: from,
            to: to,
            created_time: Utc::now().timestamp(),
            body: serde_json::json!(body),
        }
    }

    pub fn resolve(jwe: Jwe, enc_secret: EdSecret) -> Result<Jwm> {
        
        idp2p_common::anyhow::bail!("Missing");
    }

    pub fn seal(&self, sig_secret: EdSecret) -> Result<Jwe> {
        let jws = Jws::new(Jpm::from(self.clone()), sig_secret)?;
        let jwe = Jwe::encrypt(jws, self.to.clone())?;
        Ok(jwe)
    }

}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_test() {
        let from = Identity::new(&vec![], &vec![]);
        let to = Identity::new(&vec![], &vec![]);
        let jwm = Jwm::new(from, to, r#"{ "body" : "body" }"#);
        assert_eq!(jwm.from.id, "bagaaierakioikcmj4ooqw54zqsedryl7lnuubne64ga443cpkegei4xftata");
        assert_eq!(jwm.to.id, "bagaaierakioikcmj4ooqw54zqsedryl7lnuubne64ga443cpkegei4xftata");
        assert_eq!(jwm.body.as_str(), Some(r#"{ "body" : "body" }"#));
    }
}
