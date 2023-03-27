use rand::{thread_rng, RngCore};
use serde::Serialize;
use sha2::Digest;

use crate::error::SdtError;

pub(crate) fn create_random<const N: usize>() -> [u8; N] {
    let mut key_data = [0u8; N];
    let mut key_rng = thread_rng();
    key_rng.fill_bytes(&mut key_data);
    key_data
}

pub(crate) fn digest<T: Serialize>(payload: &T) -> Result<String, SdtError> {
    Ok(digest_str(&serde_json::to_string(payload)?))
}

pub(crate) fn digest_hex(payload: &str) -> String {
    to_hex_str(sha2::Sha256::digest(payload.as_bytes()))
}

pub(crate) fn to_hex_str<T: AsRef<[u8]>>(data: T) -> String{
   format!("0x{}", hex::encode(data))
}

#[derive(PartialEq, Debug, Clone)]
struct QueryNode {
    parent: Option<Box<QueryNode>>,
    path: String,
    children: Vec<QueryNode>,
}

pub fn parse_query(query: &str) -> Vec<String> {
    let mut query_keys: Vec<String> = vec![];
    let lines: Vec<&str> = query.trim().split("\n").map(|x| x.trim()).collect();
    let mut node = QueryNode {
        parent: None,
        path: "".to_string(),
        children: vec![],
    };
    for line in lines {
        if line.ends_with("{") {
            let new_node = QueryNode {
                path: format!("{}{}/", node.path, line.replace("{", "").trim()),
                parent: Some(Box::new(node.clone())),
                children: vec![],
            };
            node.children.push(new_node.clone());
            node = new_node;
        } else if line.trim() == "}" {
            node = *node.parent.unwrap();
        } else {
            query_keys.push(format!("{}{}/", node.path, line));
        }
    }
    query_keys
}


#[cfg(test)]
mod tests {
    use super::*;
 
    #[test]
    fn parse_test() {
        let query = "
            {
                personal {
                    name
                    surname
                }
            }
            ";
        let items = parse_query(query);
        assert_eq!(items, vec!["/personal/name/", "/personal/surname/"]);
    }
}
