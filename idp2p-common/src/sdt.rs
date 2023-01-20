use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::Digest;

use crate::random::create_random;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct TrieNode {
    key: String,
    proof: String,
    value: TrieNodeKind,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum TrieNodeKind {
    Masked,
    Claim {
        raw: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        salt: Option<String>,
    },
    Branch {
        children: Vec<TrieNode>,
    },
}

fn digest(payload: &str) -> String {
    hex::encode(sha2::Sha256::digest(payload.as_bytes()))
}

impl TrieNode {
    pub fn new_branch(key: &str, children: Vec<Self>) -> Self {
        let mut payload = json!({});
        for child in &children {
            payload[child.key.clone()] = serde_json::Value::String(child.proof.clone())
        }

        TrieNode {
            key: key.to_string(),
            proof: digest(&payload.to_string()),
            value: TrieNodeKind::Branch { children },
        }
    }

    pub fn new_claim(key: &str, value: &str) -> Self {
        TrieNode {
            key: key.to_string(),
            proof:  digest(value),
            value: TrieNodeKind::Claim {
                raw: value.to_string(),
                salt: None,
            },
        }
    }

    pub fn new_salted(key: &str, value: &str) -> Self {
        let salt = hex::encode(create_random::<16>());
        let payload = json!({ "raw": value, "salt": salt });
        TrieNode {
            key: key.to_string(),
            proof: digest(&payload.to_string()),
            value: TrieNodeKind::Claim {
                raw: value.to_string(),
                salt: Some(salt.to_string()),
            },
        }
    }

    pub fn revealByQuery(&self, query: &str) -> Self {
       self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn trie_test() {
        let personal = TrieNode::new_branch("personal", vec![TrieNode::new_claim("name", "Adem")]);
        let assertion_key1 = TrieNode::new_claim("key1", "11111111");
        let assertion_keys = TrieNode::new_branch("assertion_keys", vec![assertion_key1]);
        let root = TrieNode::new_branch("/", vec![personal, assertion_keys]);
        eprintln!("{}", serde_json::to_string(&root).unwrap());
        //eprintln!("{:?}", trie);
    }
}
