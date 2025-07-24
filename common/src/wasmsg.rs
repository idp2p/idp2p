use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use crate::bytes::Bytes;

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Wasmsg {
    pub id: String,
    pub protocol: String,
    #[serde_as(as = "Bytes")]
    pub body: Vec<u8>
}