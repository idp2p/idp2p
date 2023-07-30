pub mod error;
pub mod element;
pub mod proof;
pub mod query;
pub mod service;
pub mod value;

use std::collections::HashMap;

use error::SdtError;
use element::SdtElement;
use proof::SdtProof;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct SdtNode {
    pub context: HashMap<String, String>,
    pub payload: SdtElement,
    pub next: Option<Box<SdtNode>>,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Sdt {
    pub subject: String,
    pub inception: SdtNode,
}

impl SdtNode {
    pub fn find_current(&mut self) -> &mut Self {
        if self.next.is_none() {
            return self;
        }
        self.next.as_mut().unwrap().find_current()
    }

    pub fn select(&mut self, query: &str) -> Result<&mut Self, SdtError> {
        self.payload.select(query)?;
        if self.next.is_none() {
            return Ok(self);
        }
        self.next.as_mut().unwrap().select(query)
    }

    pub fn gen_proof(&self, prev: &str) -> Result<String, SdtError> {
        let paylaod_proof = self.payload.gen_proof()?;
        let item_proof = SdtProof::new()
            .insert_str("previous", prev)
            .insert_str("root", &paylaod_proof)
            .digest()?;
        if let Some(next) = &self.next {
            return next.gen_proof(&item_proof);
        } else {
            return Ok(item_proof);
        }
    }
}

impl Sdt {
    pub fn new(sub: &str, ctx: HashMap<String, String>, payload: SdtElement) -> Self {
        Sdt {
            subject: sub.to_owned(),
            inception: SdtNode {
                context: ctx,
                payload,
                next: None,
            },
        }
    }

    pub fn mutate(&mut self, ctx: HashMap<String, String>, payload: SdtElement) -> &mut Self {
        let current = self.inception.find_current();
        current.next = Some(Box::new(SdtNode{
            context: ctx,
            payload,
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
        let inception_root = self.inception.payload.gen_proof()?;
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
    use crate::element::SdtClaim;

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
        let context = HashMap::from([("personal".to_owned(), "baxyz".to_owned())]);
        let mut sdt = Sdt::new("did:p2p:123456", context.clone(), new_claim.to_element())
            .mutate(context.clone(), mutation.to_element())
            .mutate(context, mutation2.to_element())
            .build();
        sdt.mutate(HashMap::new(), SdtElement::new());
        eprintln!("{:?}", sdt);
        let proof = sdt.gen_proof()?;
        let selected = sdt.select(query)?;
        let proof2 = selected.gen_proof()?;
        assert_eq!(proof, proof2);
        Ok(())
    }
}
