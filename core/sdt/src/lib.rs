pub mod error;
pub mod node;
pub mod proof;
pub mod query;
pub mod service;
pub mod value;

use std::collections::HashMap;

use error::SdtError;
use node::SdtNode;
use proof::SdtProof;
use serde::{Deserialize, Serialize};

pub type SdtContext = HashMap<String, String>;
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct SdtItem {
    pub context: SdtContext,
    pub node: SdtNode,
    pub next: Option<Box<SdtItem>>,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Sdt {
    pub subject: String,
    pub inception: SdtItem,
}

impl SdtItem {
    pub fn find_current(&mut self) -> &mut Self {
        if self.next.is_none() {
            return self;
        }
        self.next.as_mut().unwrap().find_current()
    }

    pub fn select(&mut self, query: &str) -> Result<&mut Self, SdtError> {
        self.node.select(query)?;
        if self.next.is_none() {
            return Ok(self);
        }
        self.next.as_mut().unwrap().select(query)
    }

    pub fn gen_proof(&self, prev: &str) -> Result<String, SdtError> {
        let node_proof = self.node.gen_proof()?;
        let item_proof = SdtProof::new()
            .insert_str("previous", prev)
            .insert_str("root", &node_proof)
            .digest()?;
        if let Some(next) = &self.next {
            return next.gen_proof(&item_proof);
        } else {
            return Ok(item_proof);
        }
    }
}

impl Sdt {
    pub fn new(sub: &str, ctx: SdtContext, node: SdtNode) -> Self {
        Sdt {
            subject: sub.to_owned(),
            inception: SdtItem {
                context: ctx,
                node,
                next: None,
            },
        }
    }

    pub fn mutate(&mut self, ctx: SdtContext, node: SdtNode) -> &mut Self {
        let current = self.inception.find_current();
        current.next = Some(Box::new(SdtItem {
            context: ctx,
            node,
            next: None,
        }));
        self
    }

    pub fn build(&mut self) -> Self {
        self.to_owned()
    }

    pub fn select(&self, query: &str) -> Result<Sdt, SdtError> {
        let mut sdt = self.clone();
        sdt.inception.select(query)?;
        Ok(sdt)
    }

    pub fn gen_proof(&self) -> Result<String, SdtError> {
        let inception_root = self.inception.node.gen_proof()?;
        let inception_proof = SdtProof::new()
            .insert_str("subject", &self.subject)
            .insert_str("root", &inception_root)
            .digest()?;
        if let Some(next) = &self.inception.next {
            return next.gen_proof(&inception_proof);
        }
        return Ok(inception_proof);
    }

    pub fn verify(&self, proof: &str) -> Result<bool, SdtError> {
        let verified_proof = self.gen_proof()?;
        if verified_proof != proof {
            return Err(SdtError::VerificationError {
                expected: proof.to_owned(),
                actual: verified_proof,
            });
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use crate::node::SdtClaim;

    use super::*;
    #[test]
    fn sdt_test() -> Result<(), SdtError> {
        let new_claim_str = r#"{
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
        let mutation_str = r#"{
            "personal": {
               "name": null,
               "surname": null
            }
        }"#;
        let mutation2_str = r#"{
            "keys": {
                "assertions": {
                   "key-1": "0x1234...."
                }
             }
        }"#;
        let query = "
        {
            personal {
                name
            }
        }
        ";
        let new_claim: SdtClaim = serde_json::from_str(new_claim_str)?;
        let mutation: SdtClaim = serde_json::from_str(mutation_str)?;
        let mutation2: SdtClaim = serde_json::from_str(mutation2_str)?;
        let mut sdt = Sdt::new("did:p2p:123456", HashMap::new(), new_claim.to_node())
            .mutate(HashMap::new(), mutation.to_node())
            .mutate(HashMap::new(), mutation2.to_node())
            .build();
        sdt.mutate(HashMap::new(), SdtNode::new());
        let proof = sdt.gen_proof()?;
        let selected = sdt.select(query)?;
        let proof2 = selected.gen_proof()?;
        assert_eq!(proof, proof2);
        Ok(())
    }
}
