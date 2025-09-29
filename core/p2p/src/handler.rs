use idp2p_id::types::Idp2pError;

use crate::{
    idp2p::core::id_verifier,
    model::{IdEvent::*, Wasmsg, WasmsgValue::*},
};

pub fn handle(message: Vec<u8>) -> Result<(), Idp2pError> {
    let message: Wasmsg = serde_json::from_slice(&message).unwrap();
    match message.value {
        Event(id_event) => match id_event {
            Requested(_) => todo!(),
            Resolved {
                id,
                inception,
                events,
            } => {
                id_verifier::verify_inception(&inception).unwrap();
            }
            Notified(id_event_receipt) => todo!(),
        },
        Command(id_command) => todo!(),
    }
    Ok(())
}
