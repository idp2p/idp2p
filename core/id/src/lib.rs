extern crate alloc;
pub mod error;
pub mod model;
const VALID_FROM: &str = "2026-01-01T00:00:00Z";
const VERSION: &'static str = "1.0";

wit_bindgen::generate!({
    world: "idp2p-verifier"
});

struct GuestComponent;

export!(GuestComponent);

impl Guest for GuestComponent {
    fn verify_proof(proof: Vec<u8>, message: Vec<u8>) -> Result<bool, String> {
        todo!()
    }

    fn verify_inception(inception: Vec<u8>) -> Result<Vec<u8>, String> {
        todo!()
    }

    fn verify_event(event: Vec<u8>, state: Vec<u8>) -> Result<Vec<u8>, String> {
        todo!()
    }
}
