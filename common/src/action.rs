use alloc::vec::Vec;
use thiserror::Error;

#[derive(Debug, PartialEq, Clone)]
pub struct IdAction<'a> {
    // Entity specific action type
    pub kind: u16,
    // Major version number
    pub major: u16,
    // Minor version number
    pub minor: u16,
    // The payload of event
    pub payload: &'a [u8]
}

#[derive(Error, Debug)]
pub enum IdActionError {
    #[error("Input buffer too short, expected at least 6 bytes, got {0}")]
    BufferTooShort(usize),
    
    #[error("Failed to convert bytes to integer")]
    ByteConversionError,
}

impl<'a> IdAction<'a> {
    pub fn new(kind: u16, major: u16, minor: u16, payload: &'a [u8]) -> Self {
        IdAction {
            kind,
            major,
            minor,
            payload
        }
    }

    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, IdActionError> {
        if bytes.len() < 6 {
            return Err(IdActionError::BufferTooShort(bytes.len()));
        }

        let kind = u16::from_le_bytes(bytes[0..2].try_into()
            .map_err(|_| IdActionError::ByteConversionError)?);
        let major = u16::from_le_bytes(bytes[2..4].try_into()
            .map_err(|_| IdActionError::ByteConversionError)?);
        let minor = u16::from_le_bytes(bytes[4..6].try_into()
            .map_err(|_| IdActionError::ByteConversionError)?);
        let payload = &bytes[6..];

        Ok(IdAction {
            kind,
            major,
            minor,
            payload
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(6 + self.payload.len());
        bytes.extend_from_slice(&self.kind.to_le_bytes());
        bytes.extend_from_slice(&self.major.to_le_bytes());
        bytes.extend_from_slice(&self.minor.to_le_bytes());
        bytes.extend_from_slice(self.payload);
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_bytes_valid() {
        let bytes = [
            1, 0,  // kind = 1
            2, 0,  // major = 2
            3, 0,  // minor = 3
            4, 5, 6  // payload
        ];

        let action = IdAction::from_bytes(&bytes).unwrap();
        assert_eq!(action.kind, 1);
        assert_eq!(action.major, 2);
        assert_eq!(action.minor, 3);
        assert_eq!(action.payload, &[4, 5, 6]);
    }

    #[test]
    fn test_from_bytes_invalid_short() {
        let bytes = [1, 0, 2];  // Too short
        let err = IdAction::from_bytes(&bytes).unwrap_err();
        assert!(matches!(err, IdActionError::BufferTooShort(3)));
    }

    #[test]
    fn test_to_bytes() {
        let payload = [4, 5, 6];
        let action = IdAction::new(1, 2, 3, &payload);
        let bytes = action.to_bytes();
        
        assert_eq!(bytes, vec![
            1, 0,  // kind = 1
            2, 0,  // major = 2
            3, 0,  // minor = 3
            4, 5, 6  // payload
        ]);
    }

    #[test]
    fn test_roundtrip() {
        let payload = [4, 5, 6];
        let original = IdAction::new(1, 2, 3, &payload);
        let bytes = original.to_bytes();
        let reconstructed = IdAction::from_bytes(&bytes).unwrap();
        assert_eq!(original, reconstructed);
    }

    #[test]
    fn test_empty_payload() {
        let payload = [];
        let action = IdAction::new(1, 2, 3, &payload);
        let bytes = action.to_bytes();
        assert_eq!(bytes.len(), 6);
        let reconstructed = IdAction::from_bytes(&bytes).unwrap();
        assert_eq!(reconstructed.payload.len(), 0);
    }
}