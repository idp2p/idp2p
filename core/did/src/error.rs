#[derive(Debug)]
pub enum DidError {
    InvalidIdFormat,
    PayloadAndIdNotMatch { expected: Vec<u8>, actual: Vec<u8> },
    InvalidHashAlg(u64),
    Unknown,
}

