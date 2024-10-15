pub fn encode<T: serde::Serialize>(value: &T) -> anyhow::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    ciborium::ser::into_writer(&value, &mut bytes)?;
    Ok(bytes)
}

pub fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> anyhow::Result<T> {
    let value: T = ciborium::de::from_reader(bytes).map_err(anyhow::Error::msg)?;
    Ok(value)
}