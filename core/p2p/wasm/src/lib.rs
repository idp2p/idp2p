use exports::idp2p::p2p::id_handler::{Guest, IdHandlerCommand};

pub mod entry;
pub mod handler;

wit_bindgen::generate!({
    world: "idp2p-p2p",
    additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize]
});

struct GuestComponent;

export!(GuestComponent);

impl Guest for GuestComponent {
    fn handle_gossip_message(msg: Vec<u8>,) -> Result<Vec<IdHandlerCommand>, String> {
        todo!()
    }

    fn handle_req_res_message(msg: Vec<u8>,) -> Result<Vec<IdHandlerCommand>, String> {
        todo!()
    }
}