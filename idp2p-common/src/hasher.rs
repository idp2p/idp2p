pub trait  Idp2pHasher {
    fn is_hash_of(&self, content: &[u8])-> anyhow::Result<bool>;
}

impl Idp2pHasher for Multihash {
    fn is_hash_of(&self, content: &[u8])-> anyhow::Result<bool> {
        match self.code() {
            0x12 => {
                let expected = Code::Sha2_256.digest(content).to_bytes();
                Ok(expected == self.to_bytes())
            }
            _ => bail!("cid::multibase::")
        }
    }
}
