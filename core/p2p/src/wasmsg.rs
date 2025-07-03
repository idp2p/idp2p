use alloc::{string::String, vec::Vec};

use crate::{error::CommonError, utils::to_base32};

// WASM message structure
#[derive(Debug, PartialEq)]
pub struct Wasmsg {
    pub protocol: [u8; 32],
    pub payload: Vec<u8>,
}

impl Wasmsg {
    pub fn new(protocol: [u8; 32], payload: Vec<u8>) -> Self {
        Self { protocol, payload }
    }

    pub fn from_bytes(encoded: &[u8]) -> Result<Self, CommonError> {
        if encoded.len() < 33 {
            return Err(CommonError::InvalidVersionedMessage);
        }

        let version = u8::from_be_bytes(encoded[0..1].try_into().unwrap());
        if version != 0 {
            return Err(CommonError::InvalidVersionedMessage);
        }
        let protocol: [u8; 32] = encoded[1..33].try_into().unwrap();
        let payload = encoded[33..].to_vec();
        Ok(Self { protocol, payload })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(33 + self.payload.len());
        bytes.extend_from_slice(&[0u8]);
        bytes.extend_from_slice(self.protocol.as_slice());
        bytes.extend_from_slice(&self.payload);
        bytes
    }

    pub fn to_string(&self) -> String {
        to_base32(&self.to_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        let vm = Wasmsg::new([0; 32], vec![1, 2, 3]);

        let encoded = vm.to_bytes();
        let decoded = Wasmsg::from_bytes(&encoded).expect("Decoding failed");

        assert_eq!(vm, decoded);
    }

    #[test]
    fn decode_error_on_short_input() {
        let result = Wasmsg::from_bytes(&[0, 1, 2]);
        assert!(result.is_err());
    }
}

