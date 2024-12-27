use crate::error::CommonError;

pub fn encode<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, CommonError> {
    let mut bytes = Vec::new();
    ciborium::ser::into_writer(&value, &mut bytes).map_err(|_| CommonError::EncodeError)?;
    Ok(bytes)
}

pub fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, CommonError> {
    let value: T = ciborium::de::from_reader(bytes).map_err(|_| CommonError::DecodeError)?;
    Ok(value)
}
