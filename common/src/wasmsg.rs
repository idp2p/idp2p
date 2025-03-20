use alloc::{string::String, vec::Vec};

use crate::error::CommonError;

// Versioned message structure
#[derive(Debug, PartialEq)]
pub struct Wasmsg {
    id: u16,
    major: u16,
    minor: u16,
}

impl Wasmsg {
    pub fn new(id: u16, major: u16, minor: u16) -> Self {
        Self { id, major, minor }
    }
    
    pub fn from_bytes(encoded: &[u8]) -> Result<Self, CommonError> { 
        if encoded.len() < 6 {
            return Err(CommonError::InvalidVersionedMessage);
        }

        let id = u16::from_be_bytes(encoded[0..2].try_into().unwrap());
        let major = u16::from_be_bytes(encoded[2..4].try_into().unwrap());
        let minor = u16::from_be_bytes(encoded[4..6].try_into().unwrap());
    
        Ok(Self { id, major, minor })
    }

    pub fn to_bytes(&self, payload: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(4 + payload.len());
        bytes.extend_from_slice(&self.id.to_be_bytes());
        bytes.extend_from_slice(&self.major.to_be_bytes());
        bytes.extend_from_slice(&self.minor.to_be_bytes());
        bytes.extend_from_slice(payload);
        bytes
    }

    pub fn to_string(&self) -> String {
        format!("{}.{}", self.major, self.minor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        let vm = Wasmsg {
            id: 1,
            major: 5,
            minor: 10,
        };

        let encoded = vm.to_bytes(b"Adem");
        let decoded = Wasmsg::from_bytes(&encoded).expect("Decoding failed");

        assert_eq!(vm, decoded);
    }

    #[test]
    fn decode_error_on_short_input() {
        let result = Wasmsg::from_bytes(&[0, 1, 2]);
        assert!(result.is_err());
    }
}
