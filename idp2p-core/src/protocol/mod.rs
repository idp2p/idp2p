
use crate::protocol::id_message::IdentityMessage;
use serde::{Serialize, Deserialize};
use idp2p_common::encode_vec;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ClientWallet {
    pub id: String,
    pub authentication_index: u32,
    #[serde(with = "encode_vec")]
    pub ciphertext: Vec<u8>,
    pub messages: Vec<ClientWalletMessage>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ClientWalletMessage{
    pub id: String,
    #[serde(with = "encode_vec")]
    pub ciphertext: Vec<u8>,
}

pub enum ClientCommand{
    Publish(IdentityMessage),
    Subscribe(String),
    Backup(ClientWallet)
} 

pub enum  ClientCommandResult {
    Published,
    Subscribed,
    Persisted
}

pub enum NodeEvent{
    Received(IdentityMessage)
}

pub mod id_message;
pub mod node;