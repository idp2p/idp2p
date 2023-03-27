use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Number;

use crate::{
    error::SdtError,
    proof::SdtProof,
    utils::parse_query,
    value::{SdtValue, SdtValueKind},
};
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SdtClaim {
    Value(SdtValueKind),
    Node(HashMap<String, SdtClaim>),
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct SdtNode(HashMap<String, SdtNodeKind>);

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SdtNodeKind {
    Proof(String),
    Value(SdtValue),
    Node(SdtNode),
}

impl SdtClaim {
    pub fn to_node(&self) -> SdtNode {
        let mut node = SdtNode::new();
        if let SdtClaim::Node(map) = &self {
            for (k, v) in map {
                match v {
                    SdtClaim::Value(val) => {
                        node.add_value(k, val.to_owned());
                    }
                    SdtClaim::Node(_) => {
                        node.add_node(k, v.to_node());
                    }
                }
            }
        }
        return node.build();
    }
}

impl SdtNodeKind {
    pub fn gen_proof(&self) -> Result<String, SdtError> {
        match &self {
            Self::Proof(p) => Ok(p.to_owned()),
            Self::Value(value) => value.gen_proof(),
            Self::Node(children) => children.gen_proof(),
        }
    }
}

impl SdtNode {
    pub fn new() -> Self {
        let map: HashMap<String, SdtNodeKind> = HashMap::new();
        Self(map)
    }

    pub fn add_node(&mut self, key: &str, map: Self) -> &mut Self {
        self.0.insert(key.to_owned(), SdtNodeKind::Node(map));
        self
    }
    pub fn add_value(&mut self, key: &str, val: SdtValueKind) -> &mut Self {
        self.0
            .insert(key.to_owned(), SdtNodeKind::Value(SdtValue::new(val)));
        self
    }

    pub fn add_proof(&mut self, key: &str, proof: &str) -> &mut Self {
        self.0
            .insert(key.to_owned(), SdtNodeKind::Proof(proof.to_owned()));
        self
    }

    pub fn add_str_value(&mut self, key: &str, val: &str) -> &mut Self {
        self.add_value(key, SdtValueKind::String(val.to_owned()))
    }

    pub fn add_number_value(&mut self, key: &str, val: i64) -> &mut Self {
        self.add_value(key, SdtValueKind::Number(Number::from(val)))
    }

    pub fn add_bool_value(&mut self, key: &str, val: bool) -> &mut Self {
        self.add_value(key, SdtValueKind::Bool(val))
    }

    pub fn add_null_value(&mut self, key: &str) -> &mut Self {
        self.add_value(key, SdtValueKind::Null)
    }

    pub fn build(&self) -> Self {
        self.to_owned()
    }

    pub fn to_claim(&self) -> SdtClaim {
        let mut map: HashMap<String, SdtClaim> = HashMap::new();
        for (k, v) in &self.0 {
            match v {
                SdtNodeKind::Value(val) => {
                    map.insert(k.to_owned(), SdtClaim::Value(val.value.to_owned()));
                }
                SdtNodeKind::Node(inner) => {
                    map.insert(k.to_owned(), inner.to_claim());
                }
                _ => {}
            }
        }

        SdtClaim::Node(map)
    }

    pub fn gen_proof(&self) -> Result<String, SdtError> {
        let mut builder = SdtProof::new();
        for (k, v) in &self.0 {
            builder.insert_str(&k, &v.gen_proof()?);
        }
        builder.digest()
    }

    pub fn select(&mut self, query: &str) -> Result<(), SdtError> {
        let query_keys = parse_query(query);
        let mut stack: Vec<(String, &mut SdtNode)> = vec![("/".to_owned(), self)];
        while let Some((path, node)) = stack.pop() {
            let mut path_keys: HashMap<String, String> = HashMap::new();
            for (key, val) in node.0.to_owned() {
                let path_key = format!("{}{}/", path, key.to_owned());
                if !query_keys.contains(&path_key) {
                    let matched = query_keys.iter().any(|x| x.starts_with(&path_key));
                    if !matched {
                        node.add_proof(&key, &val.gen_proof()?);
                    } else {
                        path_keys.insert(key, path_key);
                    }
                }
            }

            for (key, val) in &mut node.0 {
                if let SdtNodeKind::Node(inner_node) = val {
                    if let Some(path_key) = path_keys.get(key) {
                        stack.push((path_key.to_owned(), inner_node));
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof() -> Result<(), SdtError> {
        let result_str = r#"
            {
                "personal": {
                    "name": {
                        "salt": "0x1234567890",
                        "value": "Adem"
                    }
                },
                "keys": "0x1234567890"
            }"#;

        let r: SdtNode = serde_json::from_str(result_str)?;
        assert_eq!(
            "0x79ee471c5bb7fb0b51a9fc628f4ad7a21f8304c0ed13ee4364efbfd4ffbd85e6",
            r.gen_proof()?
        );
        Ok(())
    }

    #[test]
    fn test_full_proof() -> Result<(), SdtError> {
        let result_str = r#"
        {
            "personal": {
              "name": {
                "salt": "0x19ea4887e02f48d2c32e7d28653e9e15",
                "value": "Adem"
              },
              "surname": {
                "salt": "0x70103fe8e86b0aec46d26399b6420bd7",
                "value": "Çağlın"
              }
            },
            "phones": {
              "dial": {
                "salt": "0x28da6aca6e0ee7123c25257321b0c8cd",
                "value": "+90dial"
              },
              "cell": {
                "salt": "0x611314ad6779d5c85217ad7107ff0dab",
                "value": "+90cell"
              }
            },
            "addresses": {
              "home": {
                "zipcode": {
                  "salt": "0xcedc029019a3ac7e18e1e5992281f00c",
                  "value": 2020
                },
                "city": {
                  "salt": "0x477ddcb35182fd349c9c0b2a4793d83b",
                  "value": "homecity"
                }
              },
              "work": {
                "zipcode": {
                  "salt": "0xc0ee44ef8e96522bc33a3ac0e49f46b6",
                  "value": 2030
                },
                "city": {
                  "salt": "0x7efa4766c245beb3e9731c11adad5ab1",
                  "value": "workcity"
                }
              }
            }
          }
        "#;
        let r: SdtNode = serde_json::from_str(result_str)?;
        assert_eq!(
            "0x5ddd4d67e93ee0cb027933eb9a024770fc985964bf7770d7f9a47033bd447c37",
            r.gen_proof()?
        );
        Ok(())
    }

    #[test]
    fn test_select() -> Result<(), SdtError> {
        let personal = SdtNode::new()
            .add_str_value("name", "Adem")
            .add_str_value("surname", "Çağlın")
            .add_bool_value("over_18", true)
            .build();
        let assertion_method = SdtNode::new().add_str_value("key_1", "0x12").build();
        let keys = SdtNode::new()
            .add_node("assertion_method", assertion_method)
            .build();
        let mut root = SdtNode::new()
            .add_node("personal", personal)
            .add_node("keys", keys)
            .build();
        let query = "
        {
          personal{
             name
             surname
          }
        }";
        root.select(query)?;
        match &root.0.get("personal").unwrap() {
            SdtNodeKind::Node(personal_node) => {
                match &personal_node.0.get("name").unwrap() {
                    SdtNodeKind::Value(_) => {}
                    _ => panic!("Name should be value"),
                }
                match &personal_node.0.get("surname").unwrap() {
                    SdtNodeKind::Value(_) => {}
                    _ => panic!("Surname should be value"),
                }
                match &personal_node.0.get("over_18").unwrap() {
                    SdtNodeKind::Proof(_) => {}
                    _ => panic!("Over 18 should be proof"),
                }
            }
            _ => panic!("Personal should be node"),
        }
        match &root.0.get("keys").unwrap() {
            SdtNodeKind::Proof(_) => {}
            _ => panic!("Personal should be node"),
        }
        Ok(())
    }

    #[test]
    fn test_new_sdt_node() {
        let sdt_node = SdtNode::new();
        assert_eq!(sdt_node.0.len(), 0);
    }

    #[test]
    fn test_add_value_to_sdt_node() {
        let mut sdt_node = SdtNode::new();
        sdt_node.add_value("name", SdtValueKind::String("John".to_owned()));
        assert_eq!(sdt_node.0.len(), 1);
        let name = sdt_node.0.get("name").unwrap();
        match name {
            SdtNodeKind::Value(val) => {
                assert_eq!(val.value, SdtValueKind::String("John".to_owned()))
            }
            _ => panic!("Expected SdtNodeKind::Value"),
        }
    }

    #[test]
    fn test_add_node_to_sdt_node() {
        let mut sdt_node = SdtNode::new();
        let mut inner_node = SdtNode::new();
        inner_node.add_value("age", SdtValueKind::new_i64(30));
        sdt_node.add_node("person", inner_node);
        assert_eq!(sdt_node.0.len(), 1);
        let person = sdt_node.0.get("person").unwrap();
        match person {
            SdtNodeKind::Node(node) => {
                assert_eq!(node.0.len(), 1);
                let age = node.0.get("age").unwrap();
                match age {
                    SdtNodeKind::Value(val) => {
                        assert_eq!(val.value, SdtValueKind::new_i64(30))
                    }
                    _ => panic!("Expected SdtNodeKind::Value"),
                }
            }
            _ => panic!("Expected SdtNodeKind::Node"),
        }
    }

    #[test]
    fn test_add_proof_to_sdt_node() {
        let mut sdt_node = SdtNode::new();
        sdt_node.add_proof("name", "proof");
        assert_eq!(sdt_node.0.len(), 1);
        let proof = sdt_node.0.get("name").unwrap();
        match proof {
            SdtNodeKind::Proof(p) => assert_eq!(p, "proof"),
            _ => panic!("Expected SdtNodeKind::Proof"),
        }
    }

    #[test]
    fn test_add_str_value_to_sdt_node() {
        let mut sdt_node = SdtNode::new();
        sdt_node.add_str_value("name", "John");
        assert_eq!(sdt_node.0.len(), 1);
        let name = sdt_node.0.get("name").unwrap();
        match name {
            SdtNodeKind::Value(val) => {
                assert_eq!(val.value, SdtValueKind::String("John".to_owned()))
            }
            _ => panic!("Expected SdtNodeKind::Value"),
        }
    }

    #[test]
    fn test_add_number_value_to_sdt_node() {
        let mut sdt_node = SdtNode::new();
        sdt_node.add_number_value("age", 30);
        assert_eq!(sdt_node.0.len(), 1);
        let age = sdt_node.0.get("age").unwrap();
        match age {
            SdtNodeKind::Value(val) => {
                assert_eq!(val.value, SdtValueKind::new_i64(30))
            }
            _ => panic!("Expected SdtNodeKind::Value"),
        }
    }
}
