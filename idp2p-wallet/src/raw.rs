use idp2p_common::anyhow::Result;
use idp2p_common::encode_vec;
use idp2p_didcomm::vcs::VerifiableCredential;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Connection {
    pub id: String,
    pub name: String,
    pub photo: Vec<u8>,
    pub accepted: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct SharedWallet {
    pub next_index: u32,
    pub next_secret_index: u32,
    pub recovery_secret_index: u32,
    pub assertion_secret_index: u32,
    pub authentication_secret_index: u32,
    pub agreement_secret_index: u32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct RawWallet {
    pub id: String,
    pub name: String,
    #[serde(with = "encode_vec")]
    pub photo: Vec<u8>,
    #[serde(with = "encode_vec")]
    pub salt: Vec<u8>,
    #[serde(with = "encode_vec")]
    pub iv: Vec<u8>,
    pub shared: SharedWallet,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub requests: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub connections: Vec<Connection>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub credentials: Vec<VerifiableCredential>,
}



impl Connection {
    pub fn new(id: &str, name: &str, photo: &[u8]) -> Self {
        Connection {
            id: id.to_owned(),
            name: name.to_owned(),
            photo: photo.to_owned(),
            accepted: false,
        }
    }

    pub fn accept(&mut self) {
        self.accepted = true;
    }
}

impl RawWallet {
    pub fn new(name: &str, photo: &[u8], id: &str, shared: SharedWallet) -> Result<Self> {
        let iv = idp2p_common::create_random::<12>();
        let salt = idp2p_common::create_random::<16>();
        let raw_wallet = RawWallet {
            id: id.to_owned(),
            name: name.to_owned(),
            photo: photo.to_owned(),
            shared: shared,
            iv: iv.to_vec(),
            salt: salt.to_vec(),
            requests: vec![],
            connections: vec![],
            credentials: vec![],
        };
        Ok(raw_wallet)
    }

    pub fn add_request(&mut self, id: &str) {
        self.requests.push(id.to_owned());
    }

    pub fn remove_request(&mut self, id: &str) {
        if let Some(index) = self.requests.iter().position(|value| *value == id) {
            self.requests.swap_remove(index);
        }
    }

    pub fn add_conn(&mut self, conn: Connection) {
        self.connections.push(conn);
    }

    pub fn accept_conn(&mut self, id: &str) {
        let conn = self
            .connections
            .iter_mut()
            .find(|conn| conn.id == id)
            .unwrap();
        conn.accept();
    }

    pub fn get_conn(&self, id: &str) -> Option<Connection> {
        self.connections
            .clone()
            .into_iter()
            .find(|conn| conn.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use idp2p_common::ed_secret::EdSecret;
    use idp2p_core::did::Identity;

    #[test]
    fn new_wallet_test() {
        let did = Identity::from_secret(EdSecret::new());
        let w = create_raw_wallet(did.id.as_str(), "adem");
        assert_eq!(w.id, did.id);
    }

    #[test]
    fn add_conn_test() {
        let did = Identity::from_secret(EdSecret::new());
        let did2 = Identity::from_secret(EdSecret::new());
        let mut w = create_raw_wallet(did.id.as_str(), "adem");
        w.add_conn(Connection::new(&did2.id, "caglin", &vec![]));
        assert_eq!(w.connections[0].id, did2.id);
    }

    
    fn create_raw_wallet(id: &str, name: &str) -> RawWallet {
        RawWallet {
            id: id.to_owned(),
            name: name.to_owned(),
            photo: vec![],
            iv: vec![],
            salt: vec![],
            requests: vec![],
            connections: vec![],
            credentials: vec![],
            shared: SharedWallet {
                next_index: 0,
                next_secret_index: 0,
                recovery_secret_index: 0,
                assertion_secret_index: 0,
                authentication_secret_index: 0,
                agreement_secret_index: 0,
            },
        }
    }
}
