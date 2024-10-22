use alloc::{ string::{String, ToString}, collections::BTreeMap };
use idp2p_core::{hash, to_hex_str};
use serde::{Deserialize, Serialize};
use crate::{ value::SdtValueKind};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct SdtProof(BTreeMap<String, SdtValueKind>);

impl SdtProof {
    pub fn new() -> Self {
        let body: BTreeMap<String, SdtValueKind> = BTreeMap::new();
        Self(body)
    }

    pub fn insert(&mut self, key: &str, value: SdtValueKind) -> &mut Self {
        self.0.insert(key.to_string(), value);
        self
    }

    pub fn insert_str(&mut self, key: &str, s: &str) -> &mut Self {
        self.insert(key, SdtValueKind::String(s.to_string()))
    }

    pub fn insert_i64(&mut self, key: &str, v: i64) -> &mut Self {
        self.insert(key, SdtValueKind::new_i64(v))
    }

    pub fn digest(&mut self) -> anyhow::Result<String> {
        let encoded = serde_json::to_vec(&self.0)?;
        let digest = hash::sha256_hash(&encoded)?;
        Ok(to_hex_str(&digest))
    }
}