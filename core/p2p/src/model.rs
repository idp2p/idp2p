use alloc::collections::BTreeSet;
use idp2p_id::types::{IdEventReceipt, IdState};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdEntry {
    pub id: String,
    pub providing: bool,
    pub state: IdState,
    pub inception: IdEventReceipt,
    pub events: BTreeSet<IdEventReceipt>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Wasmsg {
    pub protocol: String,
    pub version: String,
    pub r#type: String,
    pub value: WasmsgValue,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum WasmsgValue {
    Command(IdCommand),
    Event(IdEvent),
}


#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum IdCommand {
    Create {
        id: String,
        inception: IdEventReceipt,
        events: BTreeSet<IdEventReceipt>,
    },
    Update(IdEventReceipt),
    Resolve(String),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum IdEvent {
    Requested(String),
    Resolved {
        id: String,
        inception: IdEventReceipt,
        events: BTreeSet<IdEventReceipt>,
    },
    Notified(IdEventReceipt),
}
