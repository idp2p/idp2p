use std::collections::BTreeMap;
use idp2p_utils::digest;
use serde::{Deserialize, Serialize};
use crate::{error::SdtError, value::SdtValueKind};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct SdtProof(BTreeMap<String, SdtValueKind>);

impl SdtProof {
    pub fn new() -> Self {
        let body: BTreeMap<String, SdtValueKind> = BTreeMap::new();
        Self(body)
    }

    pub fn insert(&mut self, key: &str, value: SdtValueKind) -> &mut Self {
        self.0.insert(key.to_owned(), value);
        self
    }

    pub fn insert_str(&mut self, key: &str, s: &str) -> &mut Self {
        self.insert(key, SdtValueKind::String(s.to_owned()))
    }

    pub fn insert_i64(&mut self, key: &str, v: i64) -> &mut Self {
        self.insert(key, SdtValueKind::new_i64(v))
    }

    pub fn digest(&mut self) -> Result<String, SdtError> {
        Ok(digest(&self.0)?)
    }
}
