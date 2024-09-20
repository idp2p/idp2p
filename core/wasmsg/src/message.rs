use crate::{id::DigestId, store::Store};
use alloc::collections::BTreeMap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WasmMethod {
    pub base_id: String, // Encoded message payload
    pub wasm_id: DigestId, // Keys should be queried[/users/abc]
    pub method: String, // Result identifier
}

impl FromStr for WasmMethod {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r"/wasmsg/(?<base_id>)/(?<wasm_id>)/(?<method>)").unwrap();
        let hay = "Homer J. Simpson";
        let Some(caps) = re.captures(hay) else { return };
        todo!()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WasmMessage {
    pub method: String, // Contract id and version /wasmsg/<base id>/<multibase id>/method
    pub payload: Vec<u8>, // Encoded message payload
    pub keys: Vec<String>, // Keys should be queried[/users/abc]
    pub result: DigestId, // Result identifier
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WasmInput {
    pub payload: Vec<u8>,
    pub state: BTreeMap<String, WasmDocument>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WasmOutput {
    documents: BTreeMap<String, WasmDocument>,
    messages: Vec<WasmMessage>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WasmDocument {
    events: Vec<Vec<u8>>,                   // /permits/{id} -> Create(issuer, ...)
    projections: BTreeMap<String, Vec<u8>>, // /quotas/2023/1 -> count = 5
}

pub struct MessageHandler<C: Caller> {
    pub store: Box<dyn Store>,
    pub caller: C,
}

pub trait Caller {
    fn call_method(&self, name: &str, input: WasmInput) -> WasmOutput;
}

impl<C: Caller> MessageHandler<C> {
    pub fn handle(&self, msg: WasmMessage) -> Result<(), ()> {
        let mut queue: Vec<WasmMessage> = vec![msg];
        while let Some(qmsg) = queue.pop() {
            let mut state: BTreeMap<String, WasmDocument> = BTreeMap::new();

            for key in qmsg.keys {
                let doc = self.store.get(&key).unwrap();
                state.insert(key, doc);
            }
            let input = WasmInput {
                payload: qmsg.payload,
                state: state,
            };

            let output: WasmOutput = self.caller.call_method(&qmsg.method, input);
            for (key, val) in output.documents {
                let mut current = self.store.get(&key).unwrap_or_else(|| WasmDocument {
                    events: vec![],
                    projections: BTreeMap::new(),
                });
                for event in val.events {
                    current.events.push(event);
                }

                for (pkey, pval) in val.projections {
                    current.projections.insert(pkey, pval);
                }

                self.store.put(&key, current)
            }
            for omsg in output.messages {
                queue.push(omsg);
            }
        }
        self.store.commit();
        Ok(())
    }
}

/*pub trait WasmMessageHandler {
    fn handle(input: WasmInput) -> Result<WasmOutput, PureError>;
}*/

// When a runtime gets a wrapped event

//pub type WasmResult = Result<BTreeMap<String, Vec<u8>>, PureError>;

/*
#[derive(Serialize, Deserialize, Debug)]
pub struct WasmOutput {
    pub events: Vec<WasmEventKind>
}
  #[purewasm_bindgen]
  pub fn handle(input: WasmInput) -> Result<Vec<WasmEventKind>, i32> {

  }
  let store = Store;
  while let Some(msg) = queue.pop() {
     let mut state: BTreeMap<String, Vec<WasmEventKind>> = BTreeMap::new();

     for key in msg.keys {
        let doc = store.get(key)?;
        state.insert(key, doc);
     }
     let input = WasmInput::new(payload, state);
     let output: WasmOutput = call_purewasm(input)?;
     for (key, events) in output.events {
        let mut current = store.get_mut(key).or(WasmValue::new());
        for event in events {
           match event {
              WasmEventKind::Event(e) => current.events.push(e),
              WasmEventKind::Projection(pkey, pval) => current.projections.insert(pkey, pval),
           }
        }

        store.put(key, current)
     }
     for omsg in output.messages {
        queue.push(omsg);
     }
  }
  store.commit()

  A channel has:
   - schemas -> schema id and index

   Genesis block
   - users -> network, signer
   - meta, algorithms
   - stored queries
   - stored mutations
*/
