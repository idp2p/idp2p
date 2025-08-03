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
    fn handle(message: Vec<u8>, state: Option<Vec<u8>>) -> Result<Vec<u8>, String> {
        todo!()
    }
}
