use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct Idp2pErrorParam {
    pub key: String,
    pub value: String
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct Idp2pError {
    pub code: String,
    pub message: String,
    pub details: Vec<Idp2pErrorParam>
}