use idp2p_id::types::IdProof;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdMessage {
    pub id: String,
    pub from: String,
    pub to: Vec<String>, // If empty for all followers
    pub payload: Vec<u8>,
    pub proof: IdProof
}