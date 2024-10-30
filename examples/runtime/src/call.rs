use wasmtime::{Caller, Extern, Memory, TypedFunc};

pub trait Host {
    fn call(&self, method: String, input: &[u8]) -> Result<Vec<u8>, String>;
}

pub fn call_fn(
    mut caller: Caller<'_, ()>,
    host: &impl Host,
    method_ptr: i32,
    method_len: i32,
    input_ptr: i32,
    input_len: i32,
) -> (i32, i32) {
    let memory: Memory = match caller.get_export("memory") {
        Some(Extern::Memory(mem)) => mem,
        _ => panic!("`memory` export not found"),
    };
    let alloc = match caller.get_export("alloc") {
        Some(Extern::Func(func)) => func,
        _ => panic!("`alloc` export not found"),
    };
    let method = {
        let mem_data = memory.data(&caller);
        let method = std::str::from_utf8(
            mem_data
                .get(method_ptr as u32 as usize..)
                .and_then(|arr| arr.get(..method_len as u32 as usize))
                .unwrap(),
        )
        .unwrap();
        method.to_string()
    };

    let input = {
        let mem_data = memory.data(&caller);
        mem_data
            .get(input_ptr as u32 as usize..)
            .and_then(|arr| arr.get(..input_len as u32 as usize))
            .unwrap()
    };

    let alloc_func: TypedFunc<i32, i32> = alloc.typed::<i32, i32>(&caller).unwrap();
    let value = host.call(method, input).unwrap();
    let value_len = value.len() as i32;
    let value_ptr = alloc_func.call(&mut caller, value_len).unwrap();
    let mem_data = memory.data_mut(&mut caller);

    mem_data
        .get_mut(value_ptr as usize..)
        .and_then(|s| s.get_mut(..value_len as usize))
        .unwrap()
        .copy_from_slice(&value);
    (value_ptr, value_len)
}
