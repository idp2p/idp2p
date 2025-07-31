extern crate alloc;
pub mod error;
pub mod protocol;
pub mod types;
const VALID_FROM: &str = "2026-01-01T00:00:00Z";
const VERSION: &'static str = "1.0";

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
    fn verify_proof(payload: Vec<u8>, proof: PersistedIdProof) -> Result<bool, String> {
        todo!()
    }

    fn verify_inception(inception: PersistedIdEvent) -> Result<Vec<u8>, String> {
        todo!()
    }

    fn verify_event(state: Vec<u8>, event: PersistedIdEvent) -> Result<Vec<u8>, String> {
        todo!()
    }
}
