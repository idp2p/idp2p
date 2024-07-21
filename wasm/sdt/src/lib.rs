#![no_main]
#![cfg_attr(not(test), no_std)]

#[macro_use]
extern crate alloc;

use alloc::string::{String, ToString};

#[cfg(target_arch = "wasm32")]
use lol_alloc::{FreeListAllocator, LockedAllocator};

mod element;
mod value;
mod proof;
mod query;

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

pub const ID_VERSION: (u16, u16, u16) = (0, 1, 0);
pub const BINARY_CODE: u64 = 0x55;