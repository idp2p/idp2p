use serde::{Deserialize, Serialize};

use crate::id::DigestId;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PureMessage {
    pub channel: String,          // Schema identifier, it is also a db id
    pub wasmid: String,           // Contract id and version /wasmid/<id>/<multibase id>
    pub payload: Vec<u8>,         // Encoded message payload
    pub projections: Vec<String>, // Keys should be queried
    pub result_proof: DigestId,   // Result identifier
}



// When a runtime gets a wrapped event

/*
  let state = BTreeMap<String, Vec<StoredEvent>> // query database 
  let input = WasmInput::new(payload, state);
  let event: WasmEvent = call_purewasm(input)?;
  for (k, v) in event {
    tx.put(k, StoredEvent([index]v))
  }
  tx.commit()
  

  A channel has:

   - users
   - meta, algorithms
   - schemas -> schema id and index
   - stored queries
   - stored mutations
*/