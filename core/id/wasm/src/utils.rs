use serde::{de::DeserializeOwned, Serialize};

use idp2p_common::cbor::{decode, encode};

pub fn to_memory<T: Serialize>(t: T) -> Result<(i32, i32), String> {
    let bytes = encode(&t).map_err(|e| e.to_string())?;
    let output_ptr = bytes.as_ptr();
    let output_len = bytes.len() as i32;
    Ok((output_ptr as i32, output_len))
}

pub unsafe fn from_memory<T: DeserializeOwned>(
    ptr: *mut u8,
    len: i32,
) -> anyhow::Result<T> {
    // 'to_vec' will copy the data because slice might cause memory leak
    let bytes = core::slice::from_raw_parts(ptr, len as usize).to_vec();
    decode(&bytes)
}

pub mod wasmke {
     // Allocation function for WebAssembly
     #[no_mangle]
     pub extern "C" fn alloc(len: usize) -> *mut u8 {
         let mut byte_array: Vec<u8> = Vec::with_capacity(len);
         let ptr = byte_array.as_mut_ptr();
         core::mem::forget(ptr);
         ptr
     }
 
     // Deallocation function for WebAssembly
     #[no_mangle]
     pub extern "C" fn de_alloc(ptr: *mut u8) {
         unsafe {
             drop(Box::from_raw(ptr));
         }
     }
}