#![no_main]
#![cfg_attr(not(test), no_std)]
#[macro_use]
extern crate alloc;

use alloc::string::{String, ToString};

#[cfg(target_arch = "wasm32")]
use lol_alloc::{FreeListAllocator, LockedAllocator};

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
    path: [ "../../wit/id", "../../wit/p2p"],
    world: "idp2p:p2p/p2p",
});

struct GuestComponent;

export!(GuestComponent);

impl Guest for GuestComponent {
   
}
