use idp2p_common::wasmsg::Wasmsg;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PersistedId {
    id: String,
    inception: Wasmsg,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    events: Vec<Wasmsg>,
}

mod tests {
    use crate::did::*;

    #[test]
    fn did_encode() {
        let did = PersistedId {
            id: "bavetds76cgrdgsbcf7er4kvc4emfq".to_string(),
            inception: Wasmsg {
                id: "bavetds76cgrdgsbcf7er4kvc4emfq".to_string(),
                protocol: "/idp2p/id".to_string(),
                version: "ba3tknc6h7n7lcw".to_string(),
                body: vec![0x00, 0x07, 0x12, 0x15, 0x00, 0x00, 0x00, 0x00],
            },
            events: vec![],
        };

        let encoded = serde_json::to_string_pretty(&did).unwrap();
        println!("{}", encoded);
    }
}
