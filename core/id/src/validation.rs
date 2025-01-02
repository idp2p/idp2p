use idp2p_common::{id::Id, CBOR_CODE, ED_CODE};

#[derive(Debug)]
pub enum IdValidationError {
    InvalidIdKind,
    InvalidCodec
}

impl ToString for IdValidationError {
    fn to_string(&self) -> String {
        match self {
            IdValidationError::InvalidIdKind => "invalid id kind".to_string(),
            IdValidationError::InvalidCodec => "invalid codec".to_string(),
        }
    }
}

pub trait IdValidator {
    fn ensure_id(&self) -> Result<(), IdValidationError>;
    fn ensure_event(&self) -> Result<(), IdValidationError>;
    fn ensure_signer(&self) -> Result<(), IdValidationError>;
}

impl IdValidator for Id {
    fn ensure_id(&self) -> Result<(), IdValidationError> {
        if self.cid.codec() != CBOR_CODE {
            return Err(IdValidationError::InvalidCodec);
        }
        match self.kind.as_str() {
            "id" => Ok(()),
            _ => Err(IdValidationError::InvalidIdKind),
        }
    }

    fn ensure_event(&self) -> Result<(), IdValidationError> {
        if self.cid.codec() != CBOR_CODE {
            return Err(IdValidationError::InvalidCodec);
        }
        match self.kind.as_str() {
            "event" => Ok(()),
            _ => Err(IdValidationError::InvalidIdKind),
        }
    }

    fn ensure_signer(&self) -> Result<(), IdValidationError> {
        if self.cid.codec() != ED_CODE {
            return Err(IdValidationError::InvalidCodec);
        }
        match self.kind.as_str() {
            "signer" => Ok(()),
            _ => Err(IdValidationError::InvalidIdKind),
        }
    }
    
}

