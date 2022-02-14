use crate::jwm::Jwm;
use idp2p_common::anyhow::{Result, *};
use idp2p_common::base64url;
use idp2p_common::{chrono::Utc, serde_json};
use serde::{Deserialize, Serialize};

const M_TYPE: &str = "https://idp2p.github.io";
const TYP: &str = "application/didcomm-plain+json";
// Json plain message
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
    pub fn from(jwm: Jwm) -> Self {
        Jpm {
            id: jwm.id.clone(),
            m_type: M_TYPE.to_owned(),
            typ: TYP.to_owned(),
            from: jwm.from.id.to_owned(),
            to: vec![jwm.to.id.to_owned()],
            created_time: Utc::now().timestamp(),
            body: jwm.body,
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        let jpm: Jpm = base64url::decode(s)?;
        if jpm.m_type != M_TYPE {
            bail!("Message type should be {}", M_TYPE)
        }
        if jpm.typ != TYP {
            bail!("Type should be {}", TYP)
        }
        Ok(jpm)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use idp2p_core::did::Identity;
    #[test]
    fn from_test() {
        let from = Identity::new(&vec![], &vec![]);
        let to = Identity::new(&vec![], &vec![]);
        let jwm = Jwm::new(from, to, r#"{ "body" : "body" }"#);
        let jpm = Jpm::from(jwm);
        let expected = "bagaaierakioikcmj4ooqw54zqsedryl7lnuubne64ga443cpkegei4xftata";
        assert_eq!(jpm.from, expected);
        assert_eq!(jpm.to[0], expected);
        assert_eq!(jpm.body.as_str(), Some(r#"{ "body" : "body" }"#));
        assert_eq!(jpm.m_type, M_TYPE);
        assert_eq!(jpm.typ, TYP);
    }

    #[test]
    fn from_str_invalid_test() {
        let j = Jpm {
            id: "jwm.id".to_owned(),
            m_type: M_TYPE.to_owned(),
            typ: "TYP".to_owned(),
            from: "from".to_owned(),
            to: vec!["to".to_owned()],
            created_time: Utc::now().timestamp(),
            body: serde_json::json!({ "body" : "body" }),
        };
        let jpm_str = serde_json::to_string(&j).unwrap();
        let r = Jpm::from_str(&jpm_str);
        assert!(r.is_err());
    }

    #[test]
    fn from_str_ok_test() {
        let j = Jpm {
            id: "jwm.id".to_owned(),
            m_type: M_TYPE.to_owned(),
            typ: TYP.to_owned(),
            from: "from".to_owned(),
            to: vec!["to".to_owned()],
            created_time: Utc::now().timestamp(),
            body: serde_json::json!(r#"{ "body" : "body" }"#),
        };
        let jpm_b64 = base64url::encode(&j).unwrap();
        let r = Jpm::from_str(&jpm_b64);
      
        assert!(r.is_ok());
    }
}
