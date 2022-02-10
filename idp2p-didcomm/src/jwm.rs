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
            body: serde_json::from_str(body).unwrap(),
        }
    }

    pub fn resolve(jwe: Jwe, enc_secret: EdSecret) -> Result<Jwm> {
        idp2p_common::anyhow::bail!("Missing");
    }

    pub fn seal(&self, sig_secret: EdSecret) -> Result<Jwe> {
        Jwe::from(self.clone(), sig_secret)
    }

}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_test() {}
}
