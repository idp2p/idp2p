pub mod action;
pub mod config;
pub mod event;
pub mod inception;
pub mod model;
pub mod signer;

wit_bindgen::generate!({
    world: "idp2p-id",
    additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize]
});

struct GuestComponent;

export!(GuestComponent);

impl Guest for GuestComponent {
    fn verify_inception(inception: Vec<u8>) -> Result<Vec<u8>, IdVerifierError> {
        todo!()
    }

    fn verify_event(view: Vec<u8>, event: Vec<u8>) -> Result<Vec<u8>, IdVerifierError> {
        todo!()
    }
}
