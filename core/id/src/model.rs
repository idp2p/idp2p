use alloc::{string::String, vec::Vec};
use anyhow::bail;
use cid::Cid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct IdMultiSig {
    m: u8,
    n: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IdKeyEventKind {
    CreateAssertion {
        id: Cid,
        pk: Vec<u8>,
        valid_from: i64,
        valid_until: i64,
    },
    RevokeAssertion(Cid),
    CreateAgreement {
        id: Cid,
        pk: Vec<u8>,
        valid_from: i64,
        valid_until: i64,
    },
    RevokeAgreement(Cid),
    CreateService {
        id: String,
        r#type: String,
        service_endpoint: String
    },
    RevokeService(String),
}


impl IdMultiSig {
    pub fn verify(&self) -> Result<()> {
        if m == 0 {
            bail!("The number of required signers `m` must be greater than 0.");
        }
        if m > n {
            bail!("The number of required signers `m` should be less than or equal to the total number of signers `n`.");
        }
    }
}

impl IdKeyEventKind {
    pub fn verify(&self) -> Result<()> {
        match &self {
            IdKeyEventKind::CreateAssertion { valid_from, valid_until, .. } => {
                if *valid_from > *valid_until {
                    bail!("The `valid_from` timestamp must be less than or equal to the `valid_until` timestamp.");
                }
            }
            IdKeyEventKind::RevokeAssertion(_) => {}
            IdKeyEventKind::CreateAgreement { valid_from, valid_until, .. } => {
                if *valid_from > *valid_until {
                    bail!("The `valid_from` timestamp must be less than or equal to the `valid_until` timestamp.");
                }
            }
            IdKeyEventKind::RevokeAgreement(_) => {}
            IdKeyEventKind::CreateService { valid_from, valid_until, .. } => {
                if *valid_from > *valid_until {
                    bail!("The `valid_from` timestamp must be less than or equal to the `valid_until` timestamp.");
                }
            }
            IdKeyEventKind::RevokeService(_) => {}
        }
    }
}