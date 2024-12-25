use crate::error::IdError;

pub fn encode<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, IdError> {
    let mut bytes = Vec::new();
    ciborium::ser::into_writer(&value, &mut bytes).map_err(|_| IdError::EncodeError)?;
    Ok(bytes)
}

pub fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, IdError> {
    let value: T = ciborium::de::from_reader(bytes).map_err(|_| IdError::DecodeError)?;
    Ok(value)
}
