extern crate alloc;

use alloc::{string::String, vec::Vec};

mod key;
mod error;
mod pubsub;
mod request;
mod response;
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
        todo!()
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
