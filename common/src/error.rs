#[derive(Debug)]
pub enum IdError {
    DecodeError,
    EncodeError,
    InvalidPublicKey(Vec<u8>),
    InvalidSignature(Vec<u8>),
    SignatureVerifyError,
    Unknown
}