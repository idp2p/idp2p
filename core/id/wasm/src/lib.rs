pub unsafe extern "C" fn handle_message(ptr: *mut u8, len: i32) -> i32 {
    let message: IdMessageKind = unsafe { from_memory(ptr, len)? };
    
}