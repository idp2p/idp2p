use idp2p_common::encode_vec;
use idp2p_core::did::Identity;
use idp2p_didcomm::vcs::VerifiableCredential;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Connection {
    /// Id of connection
    pub id: String,
    /// Username of id
    pub name: String,
    /// Photo of id
    #[serde(with = "encode_vec")]
    pub photo: Vec<u8>,
    /// Sent messages
    pub sent_messages: Vec<SentMessage>,
    /// Received messages
    pub received_messages: Vec<ReceivedMessage>,
    pub accepted: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct SentMessage {
    pub text: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ReceivedMessage {
    pub text: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct RawWallet {
    pub name: String,
    pub identity: Identity,
    #[serde(with = "encode_vec")]
    pub photo: Vec<u8>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub requests: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub connections: Vec<Connection>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub credentials: Vec<VerifiableCredential>,
}

impl SentMessage {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_owned(),
        }
    }
}
impl Connection {
    pub fn new(id: &str, name: &str, photo: &[u8]) -> Self {
        Connection {
            id: id.to_owned(),
            name: name.to_owned(),
            photo: photo.to_owned(),
            sent_messages: vec![],
            received_messages: vec![],
            accepted: false,
        }
    }

    pub fn accept(&mut self) {
        self.accepted = true;
    }
}

impl RawWallet {
    pub fn new(name: &str, did: Identity, photo: &[u8]) -> Self {
        RawWallet {
            name: name.to_owned(),
            identity: did,
            photo: photo.to_vec(),
            requests: vec![],
            connections: vec![],
            credentials: vec![],
        }
    }

    pub fn add_request(&mut self, id: &str) {
        self.requests.push(id.to_owned());
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

    pub fn add_sent_message(&mut self, id: &str, message: SentMessage) {
        let conn = self
            .connections
            .iter_mut()
            .find(|conn| conn.id == id)
            .unwrap();
        conn.sent_messages.push(message);
    }

    pub fn add_received_message(&mut self, id: &str, message: ReceivedMessage) {
        let conn = self
            .connections
            .iter_mut()
            .find(|conn| conn.id == id)
            .unwrap();
        conn.received_messages.push(message);
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
        let w = RawWallet::new("adem", did.clone(), &vec![]);
        assert_eq!(w.identity, did);
    }

    #[test]
    fn add_conn_test() {
        let did = Identity::from_secret(EdSecret::new());
        let did2 = Identity::from_secret(EdSecret::new());
        let mut w = RawWallet::new("adem", did, &vec![]);
        w.add_conn(Connection::new(&did2.id, "caglin", &vec![]));
        assert_eq!(w.connections[0].id, did2.id);
    }

    #[test]
    fn add_message() {
        let did = Identity::from_secret(EdSecret::new());
        let did2 = Identity::from_secret(EdSecret::new());
        let mut w = RawWallet::new("adem", did, &vec![]);
        w.add_conn(Connection::new(&did2.id, "caglin", &vec![]));
        w.add_sent_message(&did2.id, SentMessage{text: "Heyy".to_owned()});
        assert_eq!(w.connections[0].sent_messages[0].text, "heyyy");
    }
}
