use crate::{idp2p::message::types::Idp2pError, model::{WasmsgValue::*, Wasmsg}};

pub fn handle(message: Vec<u8>) -> Result<(), Idp2pError> {
    let message: Wasmsg = serde_json::from_slice(&message).unwrap();
    match message.value {
        Event(id_event) => todo!(),
        Command(id_command) => todo!(),
    }
}