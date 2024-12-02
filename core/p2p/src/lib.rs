use exports::idp2p::p2p::id_handler::{Guest, IdHandlerEvent, IdMessageKind};

pub mod entry;
pub mod handler;

wit_bindgen::generate!({
    world: "idp2p-p2p",
    additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize]
});

struct GuestComponent;

export!(GuestComponent);

impl Guest for GuestComponent {
    fn handle_message(_kind: IdMessageKind, _msg: Vec<u8>,) -> Result<Vec<IdHandlerEvent>, String> {
        Ok(vec![])
    }
}