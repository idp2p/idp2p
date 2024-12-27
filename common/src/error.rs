#[derive(Debug)]
pub enum CommonError {
    DecodeError,
    EncodeError,
    InvalidPublicKey(Vec<u8>),
    InvalidSignature(Vec<u8>),
    SignatureVerifyError,
    InvalidIdentifier,
    Unknown
}