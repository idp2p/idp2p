extern crate alloc;

use alloc::vec::Vec;
use exports::idp2p::p2p::message_handler::Guest;
use idp2p::p2p::error::Idp2pError;

mod error;
mod message;

pub mod model;

wit_bindgen::generate!({
    world: "idp2p-p2p",
    additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize],
});

struct GuestComponent;

export!(GuestComponent);

impl Guest for GuestComponent {
    fn handle_message(payload: Vec<u8>) -> Result<(), Idp2pError> {
        todo!()
    }
}
