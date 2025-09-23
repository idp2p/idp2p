use serde::{Deserialize, Serialize};
use alloc::string::String;
use serde_json::Value;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Wasmsg {
    pub protocol: String,
    pub version: String,
    pub r#type: String,
    pub value: Value,
}