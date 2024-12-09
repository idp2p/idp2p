pub struct IdMessage {
    pub version: u64,
    pub payload: Vec<u8>,
}

impl IdMessage {
    pub fn new(version: u64, payload: Vec<u8>) -> Self {
        Self { version, payload }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.payload.len() + 8);
        bytes.extend_from_slice(self.version.to_le_bytes().as_slice());
        bytes.extend_from_slice(self.payload.as_slice());
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        let version = u64::from_le_bytes(
            bytes[0..8]
                .try_into()
                .map_err(anyhow::Error::msg)?,
        );
        let payload = bytes[8..].to_vec();
        Ok(Self { version, payload })
    }
}
