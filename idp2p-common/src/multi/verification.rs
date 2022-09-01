pub mod ed25519;
pub mod dilithium3;
pub mod winternitz;

use std::io::Read;

use super::error::Idp2pMultiError;
use unsigned_varint::{encode as varint_encode, io::read_u64};

#[derive(PartialEq, Clone, Debug)]
pub enum VerificationKeyCode{
    Ed25519 = 0xed,
    Dilithium3 =  0x1207,
    Winternitz = 0x1208
}

const ED25519_U64: u64 = VerificationKeyCode::Ed25519 as u64;
const DILITHIUM3_U64: u64 = VerificationKeyCode::Dilithium3 as u64;
const WINTERNITZ_U64: u64 = VerificationKeyCode::Winternitz as u64;
impl TryFrom<u64> for VerificationKeyCode{
    type Error = Idp2pMultiError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            ED25519_U64 => Ok(Self::Ed25519),
            DILITHIUM3_U64 => Ok(Self::Dilithium3),
            WINTERNITZ_U64 => Ok(Self::Winternitz),
            _ => Err(Idp2pMultiError::InvalidKeyCode),
        }
    }
}

impl VerificationKeyCode{
    pub fn pub_size(&self) -> u64{
        match &self {
            VerificationKeyCode::Ed25519 => 32,
            VerificationKeyCode::Dilithium3 => 1952,
            VerificationKeyCode::Winternitz => 32,
        }
    }
}
pub trait Verifier {
    fn pub_bytes(&self) -> Vec<u8>;
    fn verify(&self, payload: &[u8], sig: &[u8]) -> Result<bool, Idp2pMultiError>;
}

pub trait Signer {
    type PublicKeyType;
    fn priv_bytes(&self) -> Vec<u8>;
    fn to_public_key(&self) -> Self::PublicKeyType;
    fn sign(&self, payload: &[u8]) -> Result<Vec<u8>, Idp2pMultiError>;
}

pub fn key_to_multi_bytes(code: VerificationKeyCode, bytes: &[u8]) -> Result<Vec<u8>, Idp2pMultiError> {
    let size = code.pub_size();
    if bytes.len() as u64 != size {
        panic!("Key length is not suitable");
    }
    let mut code_buf = varint_encode::u64_buffer();
    let code = varint_encode::u64(code as u64, &mut code_buf);
    Ok([code, bytes].concat())
}

pub fn key_from_multi_bytes(bytes: &[u8]) -> Result<(VerificationKeyCode, Vec<u8>), Idp2pMultiError> {
    let mut r = bytes;
    let code: VerificationKeyCode = read_u64(&mut r)?.try_into()?;
    let size = code.pub_size();
    let mut public = vec![0u8; size as usize];
    r.read_exact(&mut public)?;
    Ok((code, public))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn enc_dec_test() -> Result<(), Idp2pMultiError> {
        let code = VerificationKeyCode::Ed25519;
        let bytes = vec![0u8;32];
        let multi_bytes = key_to_multi_bytes(code.clone(), &bytes)?;
        let (dec_code, dec_bytes) = key_from_multi_bytes(&multi_bytes)?;
        assert_eq!(code, dec_code);
        assert_eq!(bytes, dec_bytes);
        Ok(())
    }
}