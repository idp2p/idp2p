#[macro_use]
extern crate alloc;

use alloc::string::{String, ToString};
use anyhow::Ok;
use idp2p_common::cbor::{decode, encode};
use inception::PersistedIdInception;
//use event::verify_event;
//use inception::verify_inception;

mod action;
mod config;
mod event;
mod inception;
mod signer;
mod snapshot;

pub const BINARY_CODE: u64 = 0x55;
pub const VERSION: &'static str = "1.0.0";

wit_bindgen::generate!({
    world: "idp2p-id",
});

struct GuestComponent;

export!(GuestComponent);

impl Guest for GuestComponent {
    fn verify_inception(inception: Vec<u8>,) -> Result<Vec<u8>, String> {
       verify_inception(inception).map_err(|e| e.to_string())
    }

    fn verify_event(snapshot: Vec<u8>,event: Vec<u8>,) -> Result<Vec<u8>, String> {
        todo!()
    }
}

fn verify_inception(inception: Vec<u8>) -> anyhow::Result<Vec<u8>> {
    let inception: PersistedIdInception = decode(inception.as_slice())?;
    let snapshot = inception.verify()?;
    Ok(encode(&snapshot)?)
}
