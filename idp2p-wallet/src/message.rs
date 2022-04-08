#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct SentMessage {
    pub text: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ReceivedMessage {
    pub text: String,
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

#[test]
    fn add_sent_message() {
        let did = Identity::from_secret(EdSecret::new());
        let did2 = Identity::from_secret(EdSecret::new());
        let profile = IdProfile::new("adem", &vec![]);
        let profile2 = IdProfile::new("caglin", &vec![]);
        let mut w = create_raw_wallet(profile, did.id.as_str());
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
        let mut w = create_raw_wallet(profile, did.id.as_str());
        w.add_conn(Connection::new(&did2.id, profile2));
        w.add_received_message(&did2.id, "Heyy");
        assert_eq!(w.connections[0].received_messages[0].text, "Heyy");
    }
