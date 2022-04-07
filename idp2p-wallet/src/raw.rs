use idp2p_common::{anyhow::Result, ed_secret::EdSecret};
use idp2p_common::encode_vec;
use idp2p_core::{did::Identity, IdProfile};
use idp2p_didcomm::vcs::VerifiableCredential;
use serde::{Deserialize, Serialize};

use crate::derive_secret;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Connection {
    /// Id of connection
    pub id: String,
    /// Username of id
    pub profile: IdProfile,
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
    pub id: String,
    pub name: String,
    #[serde(with = "encode_vec")]
    pub photo: Vec<u8>,
    #[serde(with = "encode_vec")]
    pub salt: Vec<u8>,
    #[serde(with = "encode_vec")]
    pub iv: Vec<u8>,
    pub next_index: u32,
    pub next_secret_index: u32,
    pub recovery_secret_index: u32,
    pub assertion_secret_index: u32,
    pub authentication_secret_index: u32,
    pub agreement_secret_index: u32,
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

impl ReceivedMessage {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_owned(),
        }
    }
}

impl Connection {
    pub fn new(id: &str, profile: IdProfile) -> Self {
        Connection {
            id: id.to_owned(),
            profile: profile,
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
    pub fn new(pro: IdProfile, id: &str, index: u32) -> Result<Self> {
        let iv = idp2p_common::create_random::<12>();
        let salt = idp2p_common::create_random::<16>();
        let raw_wallet = RawWallet {
            id: id.to_owned(),
            name: pro.name,
            photo: pro.photo,
            iv: iv.to_vec(),
            salt: salt.to_vec(),
            next_index: index,
            next_secret_index: index,
            recovery_secret_index: index,
            assertion_secret_index: index,
            authentication_secret_index: index,
            agreement_secret_index: index,
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

    pub fn add_sent_message(&mut self, id: &str, msg: &str) {
        let conn = self
            .connections
            .iter_mut()
            .find(|conn| conn.id == id)
            .unwrap();
        conn.sent_messages.push(SentMessage::new(msg));
    }

    pub fn add_received_message(&mut self, id: &str, msg: &str) {
        let conn = self
            .connections
            .iter_mut()
            .find(|conn| conn.id == id)
            .unwrap();
        conn.received_messages.push(ReceivedMessage::new(msg));
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
        let profile = IdProfile::new("adem", &vec![]);
        let w = create_raw_wallet(profile, did.clone());
        assert_eq!(w.id, did.id);
    }

    #[test]
    fn add_conn_test() {
        let did = Identity::from_secret(EdSecret::new());
        let did2 = Identity::from_secret(EdSecret::new());
        let profile = IdProfile::new("adem", &vec![]);
        let profile2 = IdProfile::new("caglin", &vec![]);
        let mut w = create_raw_wallet(profile, did.clone());
        w.add_conn(Connection::new(&did2.id, profile2));
        assert_eq!(w.connections[0].id, did2.id);
    }

    #[test]
    fn add_sent_message() {
        let did = Identity::from_secret(EdSecret::new());
        let did2 = Identity::from_secret(EdSecret::new());
        let profile = IdProfile::new("adem", &vec![]);
        let profile2 = IdProfile::new("caglin", &vec![]);
        let mut w = create_raw_wallet(profile, did.clone());
        w.add_conn(Connection::new(&did2.id, profile2));
        w.add_sent_message(&did2.id, "Heyy");
        assert_eq!(w.connections[0].sent_messages[0].text, "Heyy");
    }

    #[test]
    fn add_received_message() {
        let did = Identity::from_secret(EdSecret::new());
        let did2 = Identity::from_secret(EdSecret::new());
        let profile = IdProfile::new("adem", &vec![]);
        let profile2 = IdProfile::new("caglin", &vec![]);
        let mut w = create_raw_wallet(profile, did.clone());
        w.add_conn(Connection::new(&did2.id, profile2));
        w.add_received_message(&did2.id, "Heyy");
        assert_eq!(w.connections[0].received_messages[0].text, "Heyy");
    }

    fn create_raw_wallet(profile: IdProfile, did: Identity) -> RawWallet{
        RawWallet {
            id: did.id,
            name: profile.name,
            photo: profile.photo,
            iv: vec![],
            salt: vec![],
            next_index: 0,
            next_secret_index: 0,
            recovery_secret_index: 0,
            assertion_secret_index: 0,
            authentication_secret_index:0,
            agreement_secret_index: 0,
            requests: vec![],
            connections: vec![],
            credentials: vec![],
        }
    }
}
