use utils::from_memory;

mod utils;

pub unsafe extern "C" fn verify_inception(ptr: *mut u8, len: i32) -> i32 {
    let message: String = unsafe { from_memory(ptr, len).unwrap() };
    todo!()
}

pub unsafe extern "C" fn verify_event(ptr: *mut u8, len: i32) -> i32 {
    let message: String = unsafe { from_memory(ptr, len).unwrap() };
    todo!()
}