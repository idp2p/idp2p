#![no_main]
#![cfg_attr(not(test), no_std)]
#[macro_use]
extern crate alloc;

use alloc::string::{String, ToString};

use exports::idp2p::id::verification::{Guest, PersistedIdEvent, PersistedIdInception};
#[cfg(target_arch = "wasm32")]
use lol_alloc::{FreeListAllocator, LockedAllocator};

mod command;
mod event;
mod verification;

pub const BINARY_CODE: u64 = 0x55;

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
    world: "idp2p-id",
});

struct GuestComponent;

export!(GuestComponent);

impl Guest for GuestComponent {
    fn verify_inception(inception: PersistedIdInception) -> Result<IdState, String> {
        
        todo!()
    }

    fn verify_event(state: IdState, event: PersistedIdEvent) -> Result<IdState, String> {
        todo!()
    }
}
/*
fn verify_inception(inception: IdInception) -> Result<IdState, String> {
        verify::verify_inception(inception).map_err(|e| e.to_string())
    }

    fn verify_event(state: IdState, event: IdEvent) -> Result<IdState, String> {
        verify::verify_event(state, event).map_err(|e| e.to_string())
    }

    fn create(input: IdInceptionInput) -> Result<IdCreateResult, String> {
        command::create(input).map_err(|e| e.to_string())
    }

    fn create_event(input: IdEventInput) -> Result<IdCreateEventResult, String> {
        command::create_event(input).map_err(|e| e.to_string())
    }
*/
