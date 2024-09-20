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
pub struct IdInception {
    pub version: String,
    pub timestamp: i64,
    pub m_of_n: IdMultiSig,
    pub next_signers: Vec<Cid>, // Should match n
    pub sdt_root: Cid,
    pub key_events: Vec<IdKeyEventKind>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdEvent {
    pub version: String,
    pub timestamp: i64,
    pub previous: Cid,
    pub signers: Vec<Cid>,      // Should match m
    pub next_signers: Vec<Cid>, // Should match n
    pub sdt_root: Option<Cid>,
    pub m_of_n: Option<IdMultiSig>,
    pub key_events: Option<Vec<IdKeyEventKind>>,
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
}


impl IdMultiSig {
    pub fn new(m: u8, n: u8) -> anyhow::Result<Self> {
        if m > n {
           bail!("");
        }
        Ok(Self { m, n })
    }
}

