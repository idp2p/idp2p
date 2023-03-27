use serde::{Deserialize, Serialize};

use crate::{
    error::SdtError,
    proof::SdtProof,
    utils::{create_random, to_hex_str},
};
use serde_json::Number;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SdtValueKind {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct SdtValue {
    pub salt: String,
    pub value: SdtValueKind,
}

impl SdtValue {
    pub fn new(value: SdtValueKind) -> Self {
        let salt = to_hex_str(create_random::<16>());
        Self { salt, value }
    }

    pub fn gen_proof(&self) -> Result<String, SdtError> {
        SdtProof::new()
            .insert_str("salt", &self.salt)
            .insert("value", self.value.clone())
            .digest()
    }
}

impl SdtValueKind {
    pub fn new_i64(number: i64) -> Self {
        SdtValueKind::Number(Number::from(number))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gen_proof_test() -> Result<(), SdtError> {
        let val = SdtValue {
            salt: "0x1234567890".to_owned(),
            value: SdtValueKind::Null,
        };
        assert_eq!(
            "0x5e92bb6b8e3d152843a08cddb5b4015ffeeb3d939ee253aadcc7ed322a7de10c",
            val.gen_proof()?
        );
        Ok(())
    }
}
