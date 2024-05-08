use alloc::vec::Vec;
use crate::serde_ext::serde_bytes_array;
use serde::{Deserialize, Serialize};

mod wots;

const WOTS_CODE: u64 = 0x1208;
const WOTS_PUBKEY_SIZE: usize = 32;
const WOTS_SIG_SIZE: usize = 1340;
const ED25519_PUBKEY_SIZE: usize = 32;
const ED25519_SIG_SIZE: usize = 64;

#[derive(Serialize, Deserialize, Debug)]
pub enum IdPublicKeyKind {
    Winternitz([u8; WOTS_PUBKEY_SIZE]),
    Ed25519([u8; ED25519_PUBKEY_SIZE]),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum IdSignatureKind {
    #[serde(with = "serde_bytes_array")]
    Winternitz([u8; WOTS_SIG_SIZE]),
    #[serde(with = "serde_bytes_array")]
    Ed25519([u8; ED25519_SIG_SIZE]),
}

impl IdPublicKeyKind {
    pub fn verify(&self, content: &[u8], sig: IdSignatureKind) -> Result<bool, &'static str> {
        match &self {
            IdPublicKeyKind::Winternitz(pk) => {
                if let IdSignatureKind::Winternitz(sig) = sig {
                    return wots::verify(pk, content, &sig);
                }
                return Err("SIG_PUBLIC_MATCH");
            }
            IdPublicKeyKind::Ed25519(_) => unimplemented!(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            IdPublicKeyKind::Winternitz(bytes) => {
                let mut encoded = [0u8; WOTS_PUBKEY_SIZE + 8];
                let (left, right) = encoded.split_at_mut(8);
                left.copy_from_slice(&WOTS_CODE.to_be_bytes());
                right.copy_from_slice(bytes);
                encoded.to_vec()
            }
            IdPublicKeyKind::Ed25519(_) => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn to_bytes_test() {
        let w = IdPublicKeyKind::Winternitz([5u8; 32]);
        let expected = [
            0, 0, 0, 0, 0, 0, 18, 8, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
            5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
        ];
        assert_eq!(expected.to_vec(), w.to_bytes());
    }
}
