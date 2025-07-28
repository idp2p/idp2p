extern crate alloc;
pub mod error;
//pub mod inception;
pub mod did;
pub mod state;
pub mod types;
pub mod handler;
const RELEASE_DATE: i64 = 1577836800; // unix timestamp in seconds(UTC) 2025-01-01;
const VERSION: &'static str = "1.0.0";



wit_bindgen::generate!({
    world: "idp2p-wasmsg",
    additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize],
    with: {
        "idp2p:wasmsg/types": idp2p_common::wasmsg,
    }
});

struct GuestComponent;

export!(GuestComponent);

impl Guest for GuestComponent {
    #[doc = "Handle the message"]
    fn handle(input: Vec<u8>) -> Result<Vec<u8>, String> {
        
        todo!()
        //Ok(crate::inception::verify(&inception).map_err(|e| e.to_string())?)
    }
}
