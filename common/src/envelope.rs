use alloc::string::String;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct WasmEnvelope {
    pub protocol: String,
    pub version: String,
    pub r#type: String,
    pub value: Value,
}

mod tests {
    use super::*;
    use alloc::string::ToString;
    use serde_json::json;

    #[test]
    fn test() {
        let env = WasmEnvelope {
            protocol: "id".to_string(),
            version: "1.0".to_string(),
            r#type: "event".to_string(),
            value: json!({
                "id": "bababhjdhkdjkjdkjdkjkjddlgj",
                "payload": "bababhjdhkdjkjdkjdkjkjddlgjbababhjdhkdjkjdkjdkjkjddlgjbababhjdhkdjkjdkjdkjkjddlgjbababhjdhkdjkjdkjdkjkjddlgj",
                "signatures": [
                    {
                        "key_id": "bababhjdhkdjkjdkjdkjkjddlgj",
                        "signature": "bababhjdhkdjkjdkjdkjkjddlgjbababhjdhkdjkjdkjdkjkjddlgjbababhjdhkdjkjdkjdkjkjddlgjbababhjdhkdjkjdkjdkjkjddlgj",
                        "created_at": 0
                    }
                ],
                "proofs": [
                    {
                        "protocol": "id",
                        "version": "1.0",
                        "type": "proof",
                        "value": { 
                            "id": "bababhjdhkdjkjdkjdkjkjddlgj",
                            "purpose": "authentication",
                            "key_id": "bababhjdhkdjkjdkjdkjkjddlgj",
                            "signature": "bababhjdhkdjkjdkjdkjkjddlgjbababhjdhkdjkjdkjdkjkjddlgjbababhjdhkdjkjdkjdkjkjddlgjbababhjdhkdjkjdkjdkjkjddlgj",
                            "created_at": 0
                        }
                }
                ]
            }),
        };
        let json = serde_json::to_string_pretty(&env).unwrap();
        println!("{}", json);
    }
}
