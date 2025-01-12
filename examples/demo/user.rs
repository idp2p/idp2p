use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OtherIdUser {
    pub name: String,
    pub id: Option<String>,
    pub is_connected: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserState {
    pub username: String,
    pub id: String,
    pub peer: String,
    pub others: Vec<OtherIdUser>,
}

impl OtherIdUser {
    pub fn new(name: &str) -> OtherIdUser {
        OtherIdUser {
            name: name.to_string(),
            id: None,
            is_connected: false,
        }
    }
}
impl UserState {
    pub fn new(username: &str, id: &str, peer: &str) -> UserState {
        match username {
            "alice" => UserState {
                username: username.to_string(),
                id: id.to_string(),
                peer: peer.to_string(),
                others: vec![OtherIdUser::new("bob"), OtherIdUser::new("dog")],
            },
            "bob" => UserState {
                username: username.to_string(),
                id: id.to_string(),
                peer: peer.to_string(),
                others: vec![OtherIdUser::new("alice"), OtherIdUser::new("dog")],
            },
            "dog" => UserState {
                username: username.to_string(),
                id: id.to_string(),
                peer: peer.to_string(),
                others: vec![OtherIdUser::new("alice"), OtherIdUser::new("bob")],
            },
            _ => panic!("Unknown user"),
        }
    }

    pub fn set_other(&mut self, username: &str, id: &str) -> () {
        let other = self.others.iter_mut().find(|o| o.name == username).unwrap();
        other.id = Some(id.to_string());
        other.is_connected = true;
    }

    pub fn is_connected(&self) -> bool {
        self.others.iter().all(|o| o.is_connected)
    }
}
