use anyhow::Result;

pub struct Content {
    pub version: u64,
    pub payload: Vec<u8>,
}

impl Content {
    pub fn new(version: u64, payload: Vec<u8>) -> Self {
        Self { version, payload }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        todo!()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        todo!()
    }
}