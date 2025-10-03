use idp2p_id::types::Idp2pError;

use crate::{
    idp2p::core::{id_verifier, p2p_sender, store},
    model::{IdEntry, Wasmsg, WasmsgValue::*},
};

pub fn handle(message: Vec<u8>) -> Result<(), Idp2pError> {
    let message: Wasmsg = serde_json::from_slice(&message).unwrap();
    match message.value {
        IdPing {
            from_id,
            from_peer_id,
            from_peer_addr,
            from_inception,
            from_events,
            to_id,
        } => {}
        IdPong {
            from_id,
            from_inception,
            from_events,
            to_id,
        } => {}
        IdNotifyEvent(id_event_receipt) => {}
        IdNotifyMessage {} => {},
    }
    Ok(())
}

/*Requested { id } => {
    let entry = store::get(&id)?;
    if let Some(entry) = entry {
        let entry: IdEntry = serde_json::from_slice(&entry).map_err(|_| Idp2pError {
            code: "".into(),
            message: "".into(),
        })?;
        if entry.providing {
            //p2p_sender::send(&peer, )
        }
    }
}
Resolved {
    id,
    inception,
    events,
} => {
    if inception.id != id {
        return Err(Idp2pError {
            code: "".into(),
            message: "".into(),
        });
    }
    let mut state = id_verifier::verify_inception(&inception)?;
    for event in events {
        state = id_verifier::verify_event(&state, &event)?;
    }
}
Notified(id_event_receipt) => {}*/
