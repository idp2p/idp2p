use crate::{error::HandlePubsubMessageError, idp2p::p2p::types::P2pEvent};

pub(crate) fn handle_create_id(id: String, inception: &[u8]) -> Result<Vec<P2pEvent>, HandlePubsubMessageError>{
    // verify id and store it to a node
    // subscribe to id
    todo!()
}

pub(crate) fn handle_update_id(id: String, event: &[u8]) -> Result<Vec<P2pEvent>, HandlePubsubMessageError>{
    // verify event and store it
    // notify event
    todo!()
}

pub(crate) fn handle_resolve_id(id: String) -> Result<Vec<P2pEvent>, HandlePubsubMessageError>{
    // notify resolve
    todo!()
}

pub(crate) fn handle_send_message(id: String, msg: &[u8]) -> Result<Vec<P2pEvent>, HandlePubsubMessageError>{
    // store the message
    // notify message
    todo!()
}