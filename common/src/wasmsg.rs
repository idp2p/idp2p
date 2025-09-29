use serde::{Deserialize, Serialize};
use alloc::string::String;
use serde_json::value::RawValue;
use alloc::boxed::Box;

#[derive(Debug, Serialize, Deserialize)]
pub struct Wasmsg {
    pub protocol: String,
    pub version: String,
    pub r#type: String,
    pub value: Box<RawValue>,
}