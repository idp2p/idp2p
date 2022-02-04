use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Jwm {
    pub id: String,
    pub m_type: String,
    pub typ: String,
    pub from: String,
    pub to: String,
    pub alg: String,
    pub body: Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_test() {
        let message = Jwm{
            id: "pp".to_owned(),
            m_type: "pp".to_owned(),
            typ: "pp".to_owned(),
            from: "pp".to_owned(),
            to: "pp".to_owned(),
            alg: "pp".to_owned(),
            body: serde_json::from_str(r#"{"data": "abc"}"#).unwrap()
        };
        println!("{:?}", serde_json::to_string(&message));
    }
}
