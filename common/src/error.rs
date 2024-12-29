#[derive(Debug)]
pub enum CommonError {
    DecodeError,
    EncodeError,
    InvalidPublicKey,
    InvalidSignature,
    SignatureVerifyError,
    Unknown
}

impl ToString for CommonError {
    fn to_string(&self) -> String {
        match self {
            CommonError::DecodeError => "decode error".to_string(),
            CommonError::EncodeError => "encode error".to_string(),
            CommonError::InvalidPublicKey => "invalid public key".to_string(),
            CommonError::InvalidSignature => "invalid signature".to_string(),
            CommonError::SignatureVerifyError => "signature verify error".to_string(),
            CommonError::Unknown => "unknown error".to_string(),
        }           
    }   
}