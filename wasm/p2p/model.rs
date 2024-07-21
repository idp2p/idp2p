use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdGossipMessage {
    GetRequest(Vec<u8>),
    GetResponse {
        id: Vec<u8>,
        latest_event_id: Vec<u8>,
    },
    Notify(Vec<u8>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdGetRequest {

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdGetResponse {  

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdConnect {
   
}