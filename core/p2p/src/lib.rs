#[macro_use]
extern crate alloc;

use alloc::string::{String, ToString};

wit_bindgen::generate!({
    world: "idp2p-p2p",
});

struct GuestComponent;

export!(GuestComponent);

impl Guest for GuestComponent {
    fn handle() -> Result<bool, String> {
        let _ = vec!["".to_string()];
        todo!()
    }
}

