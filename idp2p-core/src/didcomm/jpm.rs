use std::str::FromStr;
use crate::didcomm::jwm::{Jwm, JwmBody};
use idp2p_common::anyhow::{Result, *};
use idp2p_common::base64url;
use idp2p_common::chrono::Utc;
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
    pub body: JwmBody,
}

impl From<Jwm> for Jpm {
    fn from(jwm: Jwm) -> Self {
        Jpm {
            id: jwm.id.clone(),
            m_type: M_TYPE.to_owned(),
            typ: TYP.to_owned(),
            from: jwm.from.to_owned(),
            to: vec![jwm.to.to_owned()],
            created_time: Utc::now().timestamp(),
            body: jwm.body,
        }
    }
}

impl FromStr for Jpm{
    type Err = idp2p_common::anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
    use crate::didcomm::jwm::JwmBody;
    #[test]
    fn from_test() {
        let body = JwmBody::Message("body".to_owned());
        let jwm = Jwm::new("from", "to", body.clone());
        let jpm = Jpm::from(jwm);
        assert_eq!(jpm.from, "from");
        assert_eq!(jpm.to[0], "to");
        assert_eq!(jpm.body, body);
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
            body: JwmBody::Message("body".to_owned()),
        };
        let jpm_b64 = base64url::encode(&j).unwrap();
        let r = Jpm::from_str(&jpm_b64);
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
            body: JwmBody::Message("body".to_owned()),
        };
        let jpm_b64 = base64url::encode(&j).unwrap();
        let r = Jpm::from_str(&jpm_b64);
        assert!(r.is_ok());
    }
}
