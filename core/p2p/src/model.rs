use idp2p_id::types::{IdEventReceipt, IdState};
use serde::{Deserialize, Serialize};
use alloc::collections::BTreeSet;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdEntry {
    pub id: String,
    pub state: IdState,
    pub inception: IdEventReceipt,
    pub events: BTreeSet<IdEventReceipt>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PeerEntry {
    pub id: String,
    pub peer_id: String,
    pub providers: BTreeSet<String>,
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
    IdRequest(String),
    IdResponse {
        id: String,
        inception: IdEventReceipt,
        events: BTreeSet<IdEventReceipt>,
    },
    Notify(IdEventReceipt),
}

/*#[derive(Debug)]
pub enum Command {
    CreateCommand {
        id: String,
        inception: IdEventReceipt,
        events: BTreeSet<IdEventReceipt>,
    },
    UpdateCommand(IdEventReceipt),
    JoinCommand(String),
    ResolveCommand(String),
}*/