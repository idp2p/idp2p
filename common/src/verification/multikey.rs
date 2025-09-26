use alloc::vec::Vec;

use crate::{error::CommonError, ED_CODE, ED_CODE_VARINT};

#[derive(Debug)]
pub struct MultiKey {
    pub codec: u64,
    pub bytes: Vec<u8>,
}

impl MultiKey {
    pub fn to_bytes(codec: u64, bytes: &[u8]) -> Vec<u8> {
        let codec = match codec {
            ED_CODE => ED_CODE_VARINT,
            _ => panic!(""),
        };
        let mut out = Vec::with_capacity(codec.len() + bytes.len());
        out.extend_from_slice(codec);
        out.extend_from_slice(bytes);
        out
    }

    pub fn from_bytes(multikey: &[u8]) -> Result<Self, CommonError> {
       todo!()
    }
}
