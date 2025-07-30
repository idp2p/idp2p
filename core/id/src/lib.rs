extern crate alloc;
pub mod error;
//pub mod inception;
//pub mod did;
//pub mod state;
//pub mod handler;
pub mod protocol;
pub mod types;
const RELEASE_DATE: &str = "2026-01-01T00:00:00Z"; // unix timestamp in seconds(UTC) 2025-01-01;
const VERSION: &'static str = "1.0.0";

wit_bindgen::generate!({
    world: "idp2p-verifier",
    additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize],
    with: {
        "idp2p:id/types": crate::types,
    }
});

struct GuestComponent;

export!(GuestComponent);

impl Guest for GuestComponent {
    fn verify_proof(event_time: String, state: Vec<u8>, proof: IdProof) -> Result<bool, String> {
        todo!()
    }

    fn verify_inception(inception: PersistedIdEvent) -> Result<_rt::Vec<u8>, String> {
        todo!()
    }

    fn verify_event(state: Vec<u8>, event: PersistedIdEvent) -> Result<Vec<u8>, String> {
        todo!()
    }
}
