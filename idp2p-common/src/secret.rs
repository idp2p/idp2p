pub trait EdKey {
    fn to_public(&self) -> [u8; 32];
    fn sign(&self) -> [u8; 32];
}

pub trait Signer<const SIG_SIZE: usize> {
    fn sign(&self) -> [u8; SIG_SIZE];
}

pub trait Hasher {}

#[derive(PartialEq, Debug, Clone)]
pub struct Idp2pSecret<const PRIV_SIZE: usize> {
    bytes: [u8; PRIV_SIZE],
}

impl<const PRIV_SIZE: usize> Idp2pSecret<PRIV_SIZE> {
    pub fn new() -> Self {
        Idp2pSecret {
            bytes: crate::create_random::<PRIV_SIZE>(),
        }
    }
    pub fn from(data: [u8; PRIV_SIZE]) -> Self {
        Idp2pSecret { bytes: data }
    }

    pub fn from_str(s: &str) -> anyhow::Result<Self> {
        let secret = Idp2pSecret {
            bytes: crate::decode_sized(s)?,
        };
        Ok(secret)
    }

    pub fn to_bytes(&self) -> [u8; PRIV_SIZE] {
        self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_test() {
        let s = Idp2pSecret::<32>::new();
        assert_eq!(s.bytes.len(), 32);
    }
}
