use idp2p_id::types::IdState;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum IdEntryKind {
    Owner,
    Client,
    Following,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdMessageDirection {
    From,
    To,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdEntry {
    pub kind: IdEntryKind,
    pub state: IdState,
    pub inception: Vec<u8>,
    pub last_event: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdMessage {
    pub id: String,
    pub from: String,
    pub to: Vec<String>,  // If empty for all followers
    pub payload: Vec<u8>, // First byte specifies encrypted or not 
    pub kid: String,
    pub proof: Vec<u8>,
}

/*#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdMessageCipher {
    pub agreement_kid: String, // Receipent agreement key id to decrypt message
    pub agreement_data: Vec<u8>, // Ephemeral public key(x25519) or ciphertext(kyber)
    pub encryption_method: u64, // AESGCM(256)
    pub encryption_iv: Vec<u8>, // 12 or 24 bytes initial vector
    pub cipherbody: Vec<u8>,   // Encrypted message body with codec
}*/