use alloc::vec::Vec;

extern crate alloc;

mod error;
mod model;
mod handler;

wit_bindgen::generate!({
    path: "../../wit/",
    world: "idp2p-message-handler",
    additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize],
     with: {
        "idp2p:core/types": idp2p_id::types,
    }
});

struct GuestComponent;

export!(GuestComponent);

impl Guest for GuestComponent {
    fn handle(message: Vec<u8>) -> Result<(), Idp2pError> {
        handler::handle(message)
    }
}
