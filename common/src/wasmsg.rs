use alloc::{string::String, vec::Vec};

use crate::error::CommonError;

// WASM message structure
#[derive(Debug, PartialEq)]
pub struct Wasmsg {
    pub major: u16,
    pub minor: u16,
}

impl Wasmsg {
    pub fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }
    
    pub fn from_bytes(encoded: &[u8]) -> Result<Self, CommonError> { 
        if encoded.len() < 5 {
            return Err(CommonError::InvalidVersionedMessage);
        }

        let _ = u16::from_be_bytes(encoded[0..1].try_into().unwrap());
        let major = u16::from_be_bytes(encoded[1..3].try_into().unwrap());
        let minor = u16::from_be_bytes(encoded[3..5].try_into().unwrap());
    
        Ok(Self { major, minor })
    }

    pub fn to_bytes(&self, payload: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(5 + payload.len());
        bytes.extend_from_slice(vec![0].as_slice());
        bytes.extend_from_slice(self.major.to_be_bytes().as_slice());
        bytes.extend_from_slice(self.minor.to_be_bytes().as_slice());
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
