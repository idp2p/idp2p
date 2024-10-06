#![no_main]
#![cfg_attr(not(test), no_std)]
#[macro_use]
extern crate alloc;

use alloc::string::{String, ToString};

#[cfg(target_arch = "wasm32")]
use lol_alloc::{FreeListAllocator, LockedAllocator};

pub const BINARY_CODE: u64 = 0x55;
pub const VERSION: &'static str = "id@1.0.0"; 

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
    path: "../../wit/id",
    world: "idp2p-id",
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
