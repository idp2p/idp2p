use unsigned_varint::io::read_u64;

use super::{decode_key_bytes, encode_key, error::Idp2pMultiError, pub_to_id};
const X25519_PUBSIZE: usize = 32;
const X25519_MULTICODE: isize = 0xec;
const KYBER512_PUBSIZE: usize = 800;
const KYBER512_MULTICODE: isize = 0x0;

pub enum Idp2pAgreementKeyCode {
    X25519 = X25519_MULTICODE,
    Kyber512 = KYBER512_MULTICODE,
}

impl TryFrom<u64> for Idp2pAgreementKeyCode {
    type Error = Idp2pMultiError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value as isize{
            X25519_MULTICODE => Ok(Self::X25519),
            _ => Err(Idp2pMultiError::InvalidKeyCode)
        }
    }
}

impl Idp2pAgreementKeyCode {
    pub fn to_pub_key(&self, bytes: &[u8]) -> Result<Idp2pAgreementPublicKey, Idp2pMultiError> {
        match &self {
            Self::X25519 => Ok(Idp2pAgreementPublicKey::X25519 {
                public: bytes.as_ref().try_into()?,
            }),
            Self::Kyber512 => Ok(Idp2pAgreementPublicKey::Kyber512 {
                public: bytes.as_ref().try_into()?,
            }),
            _ => Err(Idp2pMultiError::InvalidKeyCode),
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum Idp2pAgreementPublicKey {
    X25519 { public: [u8; X25519_PUBSIZE] },
    Kyber512 { public: [u8; KYBER512_PUBSIZE] },
}

impl Idp2pAgreementPublicKey {
    pub fn decode<T: AsRef<[u8]>>(bytes: T) -> Result<Self, Idp2pMultiError> {
        let mut r = bytes.as_ref();
        let code = read_u64(&mut r)?.try_into()?;
        match code {
            Idp2pAgreementKeyCode::X25519 => Ok(Self::X25519 {
                public: decode_key_bytes(&mut r)?,
            }),
            Idp2pAgreementKeyCode::Kyber512 => {
                todo!()
            }
            _ => Err(Idp2pMultiError::InvalidKeyCode),
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        match &self {
            Self::X25519 { public } => encode_key(Idp2pAgreementKeyCode::X25519 as u64, public),
            Self::Kyber512 { public } => todo!(),
        }
    }

    // Create 
    pub fn create_shared_secret(&self) -> Result<(Vec<u8>, Vec<u8>), Idp2pMultiError> {
        match self {
            Self::X25519 { public } => idp2p_x25519::create_ss(*public),
            Self::Kyber512 { public } => todo!(),
        }
    }

    pub fn to_id(&self) -> [u8; 16] {
        match self {
            Self::X25519 { public } => pub_to_id(public),
            Self::Kyber512 { public } => todo!(),
        }
    }
}

mod idp2p_x25519 {
    use rand::rngs::OsRng;
    use x25519_dalek::{EphemeralSecret, PublicKey};

    use crate::multi_::error::Idp2pMultiError;

    use super::Idp2pAgreementPublicKey;
    pub(super) fn create_ss(public: [u8;32]) -> Result<(Vec<u8>, Vec<u8>), Idp2pMultiError> {
        let ephemeral_secret = EphemeralSecret::new(OsRng);
        let ephemeral_public = PublicKey::from(&ephemeral_secret);
        let ephemeral_key = Idp2pAgreementPublicKey::X25519 {
            public: ephemeral_public.to_bytes(),
        };
        let pk = PublicKey::try_from(public)?;
        let shared_secret = ephemeral_secret.diffie_hellman(&pk);
        Ok((shared_secret.to_bytes().to_vec(), ephemeral_key.encode()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn enc_dec_test() -> Result<(), Idp2pMultiError> {
        let bytes = [0u8; 32];
        let key = Idp2pAgreementKeyCode::X25519.to_pub_key(&bytes)?;
        let decoded_key = Idp2pAgreementPublicKey::decode(key.encode())?;
        matches!(decoded_key, Idp2pAgreementPublicKey::X25519 { public } if public == bytes);
        Ok(())
    }

}
