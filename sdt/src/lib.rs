#![no_main]
#![cfg_attr(not(test), no_std)]

#[macro_use]
extern crate alloc;

use alloc::string::{String, ToString};

use host::rand;
#[cfg(target_arch = "wasm32")]
use lol_alloc::{FreeListAllocator, LockedAllocator};

mod element;
mod value;
mod proof;
mod query;

pub const SDT_VERSION: (u16, u16, u16) = (0, 1, 0);

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOCATOR: LockedAllocator<FreeListAllocator> =
    LockedAllocator::new(FreeListAllocator::new());

#[cfg(target_arch = "wasm32")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

wit_bindgen::generate!({
    path: ".",
    world: "sdt",
});

struct SdtComponent;

export!(SdtComponent);

impl Guest for SdtComponent {
    fn gen_sdt(raw: String,) -> Result<String, String> {
        Err(raw)
    }
        
    fn query_sdt(sdt: String, query:String,) -> Result<String, String> {
        todo!()
    }  

    fn verify_sdt(proof: String, root: String) -> Result<bool, String> {
        let x =  rand();
        todo!()
    }  
}

