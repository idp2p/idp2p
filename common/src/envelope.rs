#[derive(Debug, PartialEq)]
pub struct WasmEnvelope {
    pub version: u16,
    pub protocol: u64,
    pub method: u16,
    pub major: u16,
    pub minor: u16,
    pub patch: [u8; 32],
    pub payload: Vec<u8>,
}