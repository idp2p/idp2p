use crate::jpm::Jpm;
use crate::jwe::Jwe;
use crate::jws::Jws;
use idp2p_common::ed_secret::EdSecret;
use idp2p_common::{anyhow::Result, chrono::Utc};
use idp2p_core::did::Identity;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ConnectRequest {
    pub name: String,
    pub photo: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum JwmBody{
    #[serde(rename = "message")]
    Message(String),
    #[serde(rename = "connect")]
    Connect(ConnectRequest)
}

pub trait JwmHandler{
    fn new_jwm(&self, to: Identity, body: JwmBody) -> Jwm;
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Jwm {
    pub id: String,
    pub from: Identity,
    pub to: Identity,
    pub created_time: i64,
    pub body: JwmBody,
}

impl Jwm {
    pub fn seal(&self, sig_secret: EdSecret) -> Result<Jwe> {
        let jws = Jws::new(Jpm::from(self.clone()), sig_secret)?;
        let jwe = Jwe::encrypt(jws, self.to.clone())?;
        Ok(jwe)
    }
}

impl JwmHandler for Identity {
    fn new_jwm(&self, to: Identity, body: JwmBody) -> Jwm {
        let id = idp2p_common::encode(&idp2p_common::create_random::<32>());
        Jwm {
            id: id,
            from: self.clone(),
            to: to,
            created_time: Utc::now().timestamp(),
            body: body,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_test() {
        let from = Identity::new(&vec![], &vec![]);
        let to = Identity::new(&vec![], &vec![]);
        let jwm = from.new_jwm(to, JwmBody::Message("body".to_owned()));
        assert_eq!(
            jwm.from.id,
            "did:p2p:bagaaierakioikcmj4ooqw54zqsedryl7lnuubne64ga443cpkegei4xftata"
        );
        assert_eq!(
            jwm.to.id,
            "did:p2p:bagaaierakioikcmj4ooqw54zqsedryl7lnuubne64ga443cpkegei4xftata"
        );
        assert_eq!(jwm.body, JwmBody::Message("body".to_owned()));
    }
}
