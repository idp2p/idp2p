use anyhow::*;
use derivation_path::ChildIndex;
use hmac::{Hmac, Mac, NewMac};
use sha2::Sha512;
use std::convert::TryInto;

const IDP2P_BIP32_NAME: &str = "idp2p seed";

#[derive(PartialEq, Debug, Clone)]
pub struct ExtendedSecretKey {
    pub depth: u8,
    pub child_index: ChildIndex,
    pub secret_key: [u8; 32],
    pub chain_code: [u8; 32],
}

type HmacSha512 = Hmac<Sha512>;

impl ExtendedSecretKey {
    pub fn from_seed(seed: [u8; 16]) -> Result<Self> {
        let mut mac = HmacSha512::new_varkey(IDP2P_BIP32_NAME.as_ref()).unwrap();
        mac.update(&seed);
        let bytes = mac.finalize().into_bytes().to_vec();
        let mut chain_code = [0; 32];
        chain_code.copy_from_slice(&bytes[32..]);

        Ok(Self {
            depth: 0,
            child_index: ChildIndex::Normal(0),
            secret_key: bytes[..32].try_into()?,
            chain_code,
        })
    }

    /// Derive an extended secret key fom the current using a derivation path
    pub fn derive<P: AsRef<[ChildIndex]>>(&self, path: &P) -> Result<Self> {
        let mut path = path.as_ref().into_iter();
        let mut next = match path.next() {
            Some(index) => self.derive_child(*index)?,
            None => self.clone(),
        };
        for index in path {
            next = next.derive_child(*index)?;
        }
        Ok(next)
    }

    /// Derive a child extended secret key with an index
    pub fn derive_child(&self, index: ChildIndex) -> Result<Self> {
        if index.is_normal() {
            return Err(anyhow!("Invalid should be hardened"));
        }

        let mut mac = HmacSha512::new_varkey(&self.chain_code).unwrap();
        mac.update(&[0u8]);
        mac.update(&self.secret_key);
        mac.update(index.to_bits().to_be_bytes().as_ref());
        let bytes = mac.finalize().into_bytes();
        let mut chain_code = [0; 32];
        chain_code.copy_from_slice(&bytes[32..]);

        Ok(Self {
            depth: self.depth + 1,
            child_index: index,
            secret_key: bytes[..32].try_into()?,
            chain_code,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use derivation_path::DerivationPath;
    use idp2p_common::decode;
    use std::convert::TryInto;

    fn root(seed: &str) -> ExtendedSecretKey {
        ExtendedSecretKey::from_seed(decode(seed).try_into().unwrap()).unwrap()
    }

    #[test]
    fn from_seed_test() {
        let vector1_path: DerivationPath = "m/0'/1'/2'/2'/1000000000'".parse().unwrap();
        let node = root("f000102030405060708090a0b0c0d0e0f")
            .derive(&vector1_path)
            .unwrap();
        assert_eq!(node.depth, 5);
        assert_eq!(node.child_index, ChildIndex::Hardened(1000000000));
        let expected = [
            121, 29, 14, 253, 141, 151, 23, 206, 190, 120, 46, 147, 125, 107, 208, 230, 211, 26,
            126, 226, 171, 73, 76, 252, 161, 249, 155, 240, 101, 170, 157, 85,
        ];
        let node2 = node.derive_child(ChildIndex::hardened(1000000001).unwrap()).unwrap();
        println!("{}", idp2p_common::encode(&node.secret_key));
        println!("{}", idp2p_common::encode(&node2.secret_key));
        assert_eq!(node.chain_code.as_ref(), expected);
    }
}
