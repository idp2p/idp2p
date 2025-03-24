extern crate alloc;

mod error;
pub mod event;
pub mod inception;
pub mod types;

const TIMESTAMP: i64 = 1735689600;

wit_bindgen::generate!({
    world: "idp2p-id",
    generate_unused_types: true,
    additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize],
});

struct GuestComponent;

export!(GuestComponent);

impl Guest for GuestComponent {
    #[doc = " Verifies an initial identity inception event."]
    fn verify_inception(incepiton: Vec<u8>) -> Result<Vec<u8>, String> {
        Ok(crate::inception::verify(&incepiton[6..]).map_err(|e| e.to_string())?)
    }

    #[doc = " Verifies an identity update event against the existing identity state."]
    fn verify_event(state: Vec<u8>, event: Vec<u8>) -> Result<Vec<u8>, String> {
        todo!()
    }
}
