use alloc::vec::Vec;

extern crate alloc;

mod error;
mod model;
mod handler;

wit_bindgen::generate!({
    path: "../../wit/msg-handler/",
    world: "idp2p-message-handler",
    additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize],
});

struct GuestComponent;

export!(GuestComponent);

impl Guest for GuestComponent {
    fn handle(message: Vec<u8>) -> Result<(), Idp2pError> {
        handler::handle(message)
    }
}
