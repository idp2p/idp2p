use crate::didcomm::jpm::Jpm;
use crate::didcomm::jwe::Jwe;
use crate::didcomm::jws::Jws;
use idp2p_common::ed_secret::EdSecret;
use idp2p_common::{anyhow::Result, chrono::Utc, encode_vec};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct IdProfile {
    pub name: String,
    #[serde(with = "encode_vec")]
    pub photo: Vec<u8>,
}

impl IdProfile {
    pub fn new(name: &str, photo: &[u8]) -> Self {
        Self {
            name: name.to_owned(),
            photo: photo.to_owned(),
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum JwmBody {
    #[serde(rename = "message")]
    Message(String),
    #[serde(rename = "connect")]
    Connect(IdProfile),
    #[serde(rename = "accept")]
    Accept(IdProfile),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Jwm {
    pub id: String,
    pub from: String,
    pub to: String,
    pub created_time: i64,
    pub body: JwmBody,
}

impl Jwm {
    pub fn new(from: &str, to: &str, body: JwmBody) -> Self {
        let id = idp2p_common::encode(&idp2p_common::create_random::<32>());
        Self {
            id: id,
            from: from.to_owned(),
            to: to.to_owned(),
            created_time: Utc::now().timestamp(),
            body: body,
        }
    }

    pub fn seal(&self, sig_secret: EdSecret, to_public: [u8; 32]) -> Result<Jwe> {
        let jws = Jws::new(Jpm::from(self.clone()), sig_secret)?;
        let jwe = Jwe::encrypt(jws, &self.to, to_public)?;
        Ok(jwe)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_test() {
        let jwm = Jwm::new("from", "to", JwmBody::Message("body".to_owned()));
        assert_eq!(jwm.from, "from");
        assert_eq!(jwm.to, "to");
        assert_eq!(jwm.body, JwmBody::Message("body".to_owned()));
    }
}
