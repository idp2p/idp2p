#![no_std]
#![no_main]

extern crate alloc;
#[cfg(target_arch = "wasm32")]
use lol_alloc::{AssumeSingleThreaded, FreeListAllocator};

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOCATOR: AssumeSingleThreaded<FreeListAllocator> =
    unsafe { AssumeSingleThreaded::new(FreeListAllocator::new()) };

use alloc::{
    string::String,
    vec::{self, Vec}, boxed::Box,
};
use bincode::{config, Decode, Encode};
use core::{mem, panic::PanicInfo};
use sha2::{Digest, Sha256};

#[derive(Encode, Decode, PartialEq, Debug)]
struct Entity {
    x: f32,
    y: f32,
}

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn id(len_ptr: *mut i32) -> *mut u8  {
    let abc: Vec<u8> = Vec::new();
    let config = config::standard();

    let world = Entity { x: 0.0, y: 4.0 };

    let encoded: Vec<u8> = bincode::encode_to_vec(&world, config).unwrap();
    let hash = Sha256::digest(&encoded);
    unsafe{
        *len_ptr = hash.len() as i32;
    }
    let ptr = hash.to_vec().as_mut_ptr();
    core::mem::forget(ptr);
    ptr
}

#[no_mangle]
pub extern "C" fn de_alloc(ptr: *mut u8) {
    unsafe{
        Box::from_raw(ptr);
    }
}
// For passing input
#[no_mangle]
pub extern "C" fn alloc(len: usize) -> *mut u8 {
    let mut byte_array: Vec<u8> = Vec::with_capacity(len); // Replace with your byte array data
    let ptr = byte_array.as_mut_ptr();
    core::mem::forget(ptr);
    ptr
}

/*#[no_mangle]
pub extern "C" fn get_len(out_ptr: *mut u8) -> *mut u8 {
    let arr = [4u8;50].to_vec();
    unsafe {
        out_ptr.copy_from(arr.as_ptr(), arr.len());
        //core::ptr::copy(arr.as_ptr(), out_ptr, arr.len());
        //let allocated_capacity = alloc::alloc::Layout::from_size_align_unchecked(arr.capacity(), 1);
        //let _ = alloc::alloc::realloc(out_ptr as *mut _, allocated_capacity, arr.len());
    }

    out_ptr
}*/


