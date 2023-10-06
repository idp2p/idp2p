use alloc::collections::BTreeMap;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct StoredEvent {
    pub wasm_index: u16,
    pub event_bytes: Vec<u8>,
}

pub struct WasmInput {
    pub payload: Vec<u8>,
    pub state: BTreeMap<String, Vec<StoredEvent>>,
}

//pub type WasmResult = Result<BTreeMap<String, Vec<u8>>, PureError>;

/*
#[purewasm_bindgen]
pub fn handle(input: WasmInput) -> WasmResult {
   
}


  let state = BTreeMap<String, Vec<StoredEvent>> // query database 
  let input = WasmInput::new(payload, state);
  let event: WasmEvent = call_purewasm(input)?;
  for (k, v) in event {
    tx.put(k, StoredEvent([index]v))
  }
  tx.commit()
  
*/ 
