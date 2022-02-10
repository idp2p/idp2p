use crate::jwm::Jwm;
use serde::{Deserialize, Serialize};
use idp2p_common::{chrono::Utc, serde_json} ;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Jpm {
    pub id: String,
    #[serde(rename = "type")]
    pub m_type: String,
    pub typ: String,
    pub from: String,
    pub to: Vec<String>,
    pub created_time: i64,
    pub body: serde_json::Value,
}

impl Jpm {
    pub fn from(jwm: Jwm) -> Jpm {
        Jpm {
            id: jwm.id.clone(),
            m_type: "https://idp2p.github.io".to_owned(),
            typ: "application/didcomm-plain+json".to_owned(),
            from: jwm.from.id.to_owned(),
            to: vec![jwm.to.id.to_owned()],
            created_time: Utc::now().timestamp(),
            body: serde_json::from_str(jwm.body.as_str().unwrap()).unwrap(),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_test() {
        /*let message = Jwm {
            id: idp2p_common::encode(&idp2p_common::create_random::<32>()),
            m_type: "pp".to_owned(),
            typ: "pp".to_owned(),
            from: "pp".to_owned(),
            to: "pp".to_owned(),
            body: serde_json::from_str(r#"{"data": "abc"}"#).unwrap(),
        };
        println!("{:?}", serde_json::to_string(&message));*/
    }
}
