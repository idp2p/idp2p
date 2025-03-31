use idp2p_common::cbor;
use serde::{Deserialize, Serialize};

use crate::error::HandleRequestError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdRequestKind {
    MessageRequest {
       peer_id: String,
       message_id: String,
    },
    IdRequest{
        challenge: Vec<u8>,
        verifier: Vec<u8>,
    }
}

pub fn handle_request(msg: &[u8]) -> Result<(), HandleRequestError> {
    let request: IdRequestKind = cbor::decode(msg)?;
    match request {
        IdRequestKind::MessageRequest {
            peer_id: _peer_id,
            message_id: _message_id,
        } => {}
        IdRequestKind::IdRequest {
            challenge: _challenge,
            verifier: _verifier,
        } => {}
    }
    Ok(())
}

