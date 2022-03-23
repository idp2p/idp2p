use idp2p_core::did::Identity;
use idp2p_core::ver_cred::VerifiableCredential;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Connection {
    /// Id of connection
    pub id: String,
    /// Username of connection
    pub username: String,
    /// Sent or Recieved messages
    pub messages: Vec<Message>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Message {
    pub sent: bool,
    pub text: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct RawWallet {
    pub username: String,
    pub identity: Identity,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub connections: Vec<Connection>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub credentials: Vec<VerifiableCredential>,
}

impl RawWallet {
    pub fn new(username: &str, did: Identity) -> Self {
        RawWallet {
            username: username.to_owned(),
            identity: did,
            connections: vec![],
            credentials: vec![],
        }
    }

    pub fn add_conn(&mut self, id: &str, username: &str) {
        let connection = Connection {
            id: id.to_owned(),
            username: username.to_owned(),
            messages: vec![],
        };
        self.connections.push(connection);
    }

    pub fn add_message(&mut self, id: &str, message: &str, sent: bool) {
        let conn = self
            .connections
            .iter_mut()
            .find(|conn| conn.id == id)
            .unwrap();
        let message = Message {
            sent: sent,
            text: message.to_owned(),
        };
        conn.messages.push(message);
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

    #[test]
    fn new_wallet_test() {
        let did = Identity::from_secret(EdSecret::new());
        let w = RawWallet::new("adem", did);
        assert_eq!(w.identity, did);
    }

    #[test]
    fn add_conn_test() {
        let did = Identity::from_secret(EdSecret::new());
        let did2 = Identity::from_secret(EdSecret::new());
        let mut w = RawWallet::new("adem", did);
        w.add_conn(&did2.id, "caglin");
        assert_eq!(w.connections[0].id, did2.id);
    }

    #[test]
    fn add_message() {
        let did = Identity::from_secret(EdSecret::new());
        let did2 = Identity::from_secret(EdSecret::new());
        let mut w = RawWallet::new("adem", did);
        w.add_conn(&did2.id, "caglin");
        w.add_message(&did2.id, "heyyy", true);
        assert_eq!(w.connections[0].messages[0].text, "heyyy");
    }
}
