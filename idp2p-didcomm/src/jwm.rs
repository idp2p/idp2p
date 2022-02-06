use crate::jwe::Jwe;
use chrono::Utc;
use idp2p_common::secret::IdSecret;
use idp2p_core::did::Identity;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use anyhow::Result;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Jwm {
    pub id: String,
    pub from: Identity,
    pub to: Identity,
    pub created_time: i64,
    pub body: Value,
}

impl Jwm {
    pub fn new(from: Identity, to: Identity, body: &str) -> Jwm {
        let id = idp2p_common::encode(&idp2p_common::create_random::<32>());
        Jwm {
            id: id,
            from: from,
            to: to,
            created_time: Utc::now().timestamp(),
            body: serde_json::from_str(body).unwrap(),
        }
    }

    pub fn resolve(jwe: Jwe, enc_secret: IdSecret) -> Result<Jwm> {
        anyhow::bail!("Missing");
    }

    pub fn seal(&self, sig_secret: IdSecret) -> Result<Jwe> {
        Jwe::from(self.clone(), sig_secret)
    }

}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_test() {}
}
