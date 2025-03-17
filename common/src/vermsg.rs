use alloc::{string::String, vec::Vec};

use crate::error::CommonError;

// Versioned message structure
#[derive(Debug, PartialEq)]
pub struct VersionedMessage {
    major: u16,
    minor: u16,
    payload: Vec<u8>,
}

impl VersionedMessage {
    pub fn new(major: u16, minor: u16, payload: Vec<u8>) -> Self {
        VersionedMessage { major, minor, payload }
    }
    
    pub fn from_bytes(encoded: &[u8]) -> Result<VersionedMessage, CommonError> { 
        if encoded.len() < 4 {
            return Err(CommonError::InvalidVersionedMessage);
        }
    
        let major = u16::from_be_bytes(encoded[0..2].try_into().unwrap());
        let minor = u16::from_be_bytes(encoded[2..4].try_into().unwrap());
        let payload = encoded[4..].to_vec();
    
        Ok(VersionedMessage { major, minor, payload })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(4 + self.payload.len());
        bytes.extend_from_slice(&self.major.to_be_bytes());
        bytes.extend_from_slice(&self.minor.to_be_bytes());
        bytes.extend_from_slice(&self.payload);
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
        let vm = VersionedMessage {
            major: 5,
            minor: 10,
            payload: b"Test Payload".to_vec(),
        };

        let encoded = vm.to_bytes();
        let decoded = VersionedMessage::from_bytes(&encoded).expect("Decoding failed");

        assert_eq!(vm, decoded);
    }

    #[test]
    fn decode_error_on_short_input() {
        let result = VersionedMessage::from_bytes(&[0, 1, 2]);
        assert!(result.is_err());
    }
}
