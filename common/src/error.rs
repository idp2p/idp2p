#[derive(Debug)]
pub enum IdError {
    InvalidId,
    DecodeError,
    EncodeError,
    EnsureError {
        expected: Vec<u8>,
        actual: Vec<u8>,
    },
    InvalidHashAlg(u64),
    InvalidPublicKey(Vec<u8>),
    InvalidSignature(Vec<u8>),
    SignatureVerifyError,
    Unknown
}