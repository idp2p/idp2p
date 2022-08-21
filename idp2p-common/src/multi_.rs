pub mod agreement;
pub mod base;
pub mod error;

use std::io::Read;

use sha2::{Digest, Sha256};
use unsigned_varint::{encode as varint_encode, io::read_u64};

use self::error::Idp2pMultiError;

pub(crate) fn pub_to_id(public: &[u8]) -> [u8; 16] {
    let h = Sha256::new().chain_update(public).finalize().to_vec();
    h[0..16].try_into().expect("Conversion failed")
}

pub(crate) fn encode_key(code: u64, bytes: &[u8]) -> Vec<u8> {
    let mut code_buf = varint_encode::u64_buffer();
    let code = varint_encode::u64(code, &mut code_buf);
    let mut size_buf = varint_encode::u64_buffer();
    let size = varint_encode::u64(bytes.len() as u64, &mut size_buf);
    [code, size, bytes].concat()
}

pub(crate) fn decode_key_bytes<const S: usize>(r: &[u8]) -> Result<[u8; S], Idp2pMultiError> {
    let size = read_u64(r)?;
    if size != S as u64 {
        return Err(Idp2pMultiError::InvalidKeyCode);
    }
    let mut key_bytes = [0u8; S];
    let mut r = r;
    r.read_exact(&mut key_bytes)?;
    Ok(key_bytes)
}
