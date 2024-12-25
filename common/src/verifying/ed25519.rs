use ed25519_dalek::{Signature, VerifyingKey};

use crate::error::IdError;

const ED25519_PUBKEY_SIZE: usize = 32;
const ED25519_SIG_SIZE: usize = 64;

pub fn verify(public: &[u8], content: &[u8], sig: &[u8]) -> Result<(), IdError> {
    let public: [u8; ED25519_PUBKEY_SIZE] = public
        .try_into()
        .map_err(|_| IdError::InvalidPublicKey(public.to_vec()))?;
    let sig: [u8; ED25519_SIG_SIZE] = sig
        .try_into()
        .map_err(|_| IdError::InvalidSignature(sig.to_vec()))?;
    let pk = VerifyingKey::from_bytes(&public)
        .map_err(|_| IdError::InvalidPublicKey(public.to_vec()))?;
    let signature = Signature::from(&sig);
    pk.verify_strict(content, &signature)
        .map_err(|_| IdError::SignatureVerifyError)?;
    return Ok(());
}

#[cfg(test)]
mod tests {
    #[test]
    fn to_bytes_test() {}
}
