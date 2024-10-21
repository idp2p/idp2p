#[macro_use]
extern crate alloc;

use alloc::string::{String, ToString};
//use event::verify_event;
//use inception::verify_inception;

mod signer;
mod config;
mod inception;
//mod event;
mod action;

pub const BINARY_CODE: u64 = 0x55;
pub const VERSION: &'static str = "1.0.0"; 

wit_bindgen::generate!({
    world: "idp2p-id",
    with: { "idp2p:shared/types": generate, },
    additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize],
});

struct GuestComponent;

export!(GuestComponent);

impl Guest for GuestComponent {
    fn verify_inception(inception: PersistedIdInception) -> Result<IdSnapshot, String> {
        //verify_inception(inception).map_err(|e| e.to_string())
        todo!()
    }

    fn verify_event(snapshot: IdSnapshot, event: PersistedIdEvent) -> Result<IdSnapshot, String> {
        //verify_event(snapshot, event).map_err(|e| e.to_string())
        todo!()

    }
}
