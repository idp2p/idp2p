extern crate alloc;

mod error;
mod model;

wit_bindgen::generate!({
    path: "../../wit",
    world: "idp2p-message-handler",
    additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize],
});

struct GuestComponent;

export!(GuestComponent);

impl Guest for GuestComponent {
    fn handle(message: _rt::Vec<u8>) -> Result<(), Idp2pError> {
        let message: String = serde_json::from_slice(&message).unwrap();
        todo!()
    }
}
