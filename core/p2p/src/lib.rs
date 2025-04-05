extern crate alloc;

use alloc::{string::String, vec::Vec};

mod error;
mod key;
mod message;
mod command;

pub mod model;

use exports::idp2p::p2p::message_handler::Guest;
use idp2p::p2p::types::{P2pError, P2pEvent};

wit_bindgen::generate!({
    world: "idp2p-p2p",
    additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize],
});

struct GuestComponent;

export!(GuestComponent);

impl Guest for GuestComponent {
    fn handle_pubsub(topic: String, payload: Vec<u8>) -> Result<Vec<P2pEvent>, P2pError> {
        message::handle_pubsub_message(&topic, &payload).map_err(|e| e.into())
    }

    fn handle_request(payload: Vec<u8>) -> Result<Vec<P2pEvent>, P2pError> {
        todo!()
    }

    fn handle_response(payload: Vec<u8>) -> Result<Vec<P2pEvent>, P2pError> {
        todo!()
    }
}

/*
           Request => {
               verify_proof(proof) => signer
               From => {
                  match(signer) => {
                     Following | FollowingMediator => send_response(message + proof)
                  }
               }
               To => {
                  match(signer) => {
                     MessageTo | MessageToMediator => send_response(message + proof)
                  }
               }
            }
            Response => {}
            Pubsub { topic, payload } => {
               let entry = get_id(topic);
               match payload {
                   Resolve => {

                   }
                   Provide => {}
                   NotifyEvent => {}
                   NotifyMessage => {}
               }
            }
*/
