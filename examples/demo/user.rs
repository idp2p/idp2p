use std::collections::HashMap;

use libp2p::PeerId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OtherIdUserState {
    New,
    Connected,
    Resolved
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OtherIdUser {
    pub name: String,
    pub id: Option<String>,
    pub state: OtherIdUserState,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserState {
    pub username: String,
    pub id: String,
    pub peer: String,
    pub others: OtherIdUser,
    pub peers: HashMap<PeerId, bool>,
}

impl OtherIdUser {
    pub fn new(name: &str) -> OtherIdUser {
        OtherIdUser {
            name: name.to_string(),
            id: None,
            state: OtherIdUserState::New,
        }
    }
}
impl UserState {
    pub fn new(username: &str, id: &str, peer: &str) -> UserState {
       let other = match username {
            "alice" => OtherIdUser::new("bob"),
            "bob" => OtherIdUser::new("alice"),
            _ => panic!("Unknown user"),
        };
        UserState {
            username: username.to_string(),
            id: id.to_string(),
            peer: peer.to_string(),
            other: other,
            peers: HashMap::new(),
        }
    }

    pub fn set_other_id(&mut self, username: &str, id: &str, peer: &PeerId) -> () {
        self.peers.insert(peer.clone(), true);
        let other = self.others.iter_mut().find(|o| o.name == username).unwrap();
        other.id = Some(id.to_string());
        other.state = OtherIdUserState::Connected;
    }
}
