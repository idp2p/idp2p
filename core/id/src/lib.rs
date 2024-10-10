#[macro_use]
extern crate alloc;

use alloc::string::{String, ToString};

pub const BINARY_CODE: u64 = 0x55;
pub const VERSION: &'static str = "id@1.0.0"; 

wit_bindgen::generate!({
    world: "idp2p-id",
    with: {"idp2p:shared/types": generate, }
});

struct GuestComponent;

export!(GuestComponent);

impl Guest for GuestComponent {
    fn verify_inception(inception: PersistedIdInception) -> Result<IdSnapshot, String> {
        todo!()
    }

    fn verify_event(snapshot: IdSnapshot, event: PersistedIdEvent) -> Result<IdSnapshot, String> {
    
        todo!()
    }
}
