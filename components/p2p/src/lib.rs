extern crate alloc;

use alloc::string::{String, ToString};
use exports::idp2p::p2p::id_handler::{Guest, IdRequest, IdResponse};
use message::handle_message_inner;

mod entry;
mod message;

wit_bindgen::generate!({
    world: "idp2p-p2p",
    additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize],
    generate_all,
});

struct GuestComponent;

export!(GuestComponent);

impl Guest for GuestComponent {
    fn handle_message(request: IdRequest) -> Result<IdResponse, String> {
        handle_message_inner(request).map_err(|e| e.to_string())
    }
}
