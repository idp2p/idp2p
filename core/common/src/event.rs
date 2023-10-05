pub type WasmEvent = BTreeMap<String, Vec<u8>>; 

#[derive(Serialize, Deserialize, Debug)]
pub struct StoredEvent {
    pub wasm_index: u16,
    pub event_bytes: Vec<u8>,
}

pub struct WasmInput {
    payload: Vec<u8>,
    state: BTreeMap<String, Vec<StoredEvent>>,
}

#[purewasm_bindgen]
pub fn handle(input: WasmInput) -> Result<WasmEvent, PureError> {

}

/*
  let state = BTreeMap<String, Vec<StoredEvent>> // query database 
  let input = WasmInput::new(payload, state);
  let event: WasmEvent = call_purewasm(input)?;
  for (k, v) in event {
    tx.put(k, StoredEvent([index]v))
  }
  tx.commit()
  
*/ 
