use std::collections::HashMap;

use call::{call_fn, Host};
use wasmtime::{Caller, Engine, Func, Instance, Module, Store};

mod call;

#[derive(Clone)]
struct HostImpl {
    engine: Engine,
    modules: HashMap<String, Module>
}

impl HostImpl {
    pub fn call_wasm(&self, module: &str, func_name: &str, input: &[u8]) -> Result<(), String> {
        let mut store = Store::new(&self.engine, ());
        let host = self.clone();
        let call_func = Func::wrap(
            &mut store,
            move |caller: Caller<'_, ()>, method_ptr: i32, method_len: i32, input_ptr: i32, input_len: i32| -> (i32, i32) {  
                call_fn(caller, &host, method_ptr, method_len, input_ptr, input_len)
            },
        );

        let module = self.modules.get(module).unwrap();
        let instance = Instance::new(&mut store, &module, &[call_func.into()]).unwrap();
        let func = instance.get_typed_func::<(i32, i32), (i32, i32)>(&mut store, func_name).unwrap();
        func.call(store, (0, 0));
        Ok(())
    }
}

impl Host for HostImpl {
    fn call(&self, method: String, input: &[u8]) -> Result<Vec<u8>, String> {
        match method.as_str() {
            _ => todo!(),      
        }
    }
}

fn handle_message(engine: &Engine, module: &Module) -> Result<(), ()> {
    let mut store = Store::new(engine, ());
    let host = HostImpl {
        engine: engine.clone(),
        modules: HashMap::new(),
    };
    let get_func = Func::wrap(
        &mut store,
        move |caller: Caller<'_, ()>, method_ptr: i32, method_len: i32, input_ptr: i32, input_len: i32| -> (i32, i32) {  
            call_fn(caller, &host, method_ptr, method_len, input_ptr, input_len)
        },
    );
    Ok(())
}
