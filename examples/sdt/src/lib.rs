#[macro_use]
extern crate alloc;

use alloc::string::String;

mod element;
mod proof;
mod query;
mod value;

pub const SDT_VERSION: (u16, u16, u16) = (0, 1, 0);

wit_bindgen::generate!({
    world: "sdt",
});

struct SdtComponent;

export!(SdtComponent);

impl Guest for SdtComponent {
    fn gen_sdt(raw: String) -> Result<String, String> {
        Err(raw)
    }

    fn query_sdt(sdt: String, query: String) -> Result<String, String> {
        todo!()
    }

    fn verify_sdt(proof: String, root: String) -> Result<bool, String> {
        let x = rand();
        todo!()
    }
}
