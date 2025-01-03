use alloc::{string::ToString, vec::Vec};

use crate::error::CommonError;

pub fn encode<T: serde::Serialize>(value: &T) -> Vec<u8> {
    let mut bytes = Vec::new();
    ciborium::ser::into_writer(&value, &mut bytes).expect("Failed to serialize");
    bytes
}

pub fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, CommonError> {
    let value: T =
        ciborium::de::from_reader(bytes).map_err(|e| CommonError::DecodeError(e.to_string()))?;
    Ok(value)
}
