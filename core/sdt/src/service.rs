use crate::{error::SdtError, node::SdtClaim, Sdt};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd", content = "payload")]
pub enum SdtInput {
    Inception { subject: String, claim: SdtClaim },
    Mutation { sdt: Sdt, claim: SdtClaim },
    Selection { sdt: Sdt, query: String },
    Proof(Sdt),
    Verification { sdt: Sdt, proof: String },
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum SdtResult {
    Inception(Sdt),
    Mutation(Sdt),
    Selection(Sdt),
    Proof(String),
    Verification(bool),
    Error(String),
}

pub struct SdtService(pub String);

impl SdtService {
    pub fn execute(&self) -> String {
        let res = match self.execute_inner() {
            Ok(res) => res,
            Err(err) => SdtResult::Error(err.to_string()),
        };
        match serde_json::to_string_pretty(&res) {
            Ok(s) => s,
            Err(e) => format!(r#"{{"kind": "Error","message": {}}}"#, e.to_string()),
        }
    }

    fn execute_inner(&self) -> Result<SdtResult, SdtError> {
        let input: SdtInput = serde_json::from_str(&self.0)?;
        let result = match input {
            SdtInput::Inception { subject, claim } => {
                SdtResult::Inception(Sdt::new(&subject, claim.to_node()))
            }
            SdtInput::Mutation { sdt, claim } => {
                let mut sdt_clone = sdt.clone();
                SdtResult::Mutation(sdt_clone.mutate(claim.to_node()).build())
            }
            SdtInput::Selection { sdt, query } => {
                let mut sdt_clone = sdt.clone();
                sdt_clone.select(&query)?;
                SdtResult::Selection(sdt_clone.build())
            }
            SdtInput::Proof(sdt) => SdtResult::Proof(sdt.gen_proof()?),
            SdtInput::Verification { sdt, proof } => SdtResult::Verification(sdt.verify(&proof)?),
        };
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_test() -> Result<(), SdtError> {
        let claim_str = r#"{
            "personal": {
               "name": "Adem",
               "surname": "Çağlın",
               "age": 5
            },
            "keys": {
               "assertions": {
                  "key-1": "0x12...."
               }
            }
        }"#;
        let claim: SdtClaim = serde_json::from_str(claim_str)?;
        let cmd = SdtInput::Inception {
            subject: "did:p2p:123456".to_owned(),
            claim: claim,
        };
        let svc = SdtService(serde_json::to_string(&cmd)?);
        let result_str = svc.execute();
        let result: SdtResult = serde_json::from_str(&result_str)?;
        assert!(matches!(result, SdtResult::Inception(_)));
        Ok(())
    }
}
