use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Mutex;

pub struct ClientStore {
    pub state: Mutex<ClientState>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct ClientState {
    pub clients: HashMap<String, Client>,
    pub subscriptions: HashSet<String>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Client {
    pub id: String,
    pub subscriptions: HashSet<String>,
}

impl ClientStore {
    pub fn new(clients: HashMap<String, Client>, subscriptions: HashSet<String>) -> Self {
        let state = ClientState {
            clients: clients,
            subscriptions: subscriptions,
        };
        ClientStore {
            state: Mutex::new(state),
        }
    }
}
