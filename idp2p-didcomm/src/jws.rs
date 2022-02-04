use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Jws {
    pub payload: String,
    pub signatures: Vec<JwsSignature>,
}

struct JwsSignature{
   pub protected: String,
   pub signature: String,
   pub header: HashMap<String, String>
}
