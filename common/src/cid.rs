use cid::{multibase::Base, Cid};
use multihash::Multihash;

use crate::{utils::sha256_hash, SHA2_256_CODE};

pub trait CidExt {
    fn ensure(&self, input: &[u8]) -> anyhow::Result<()>;
    fn create(code: u64, input: &[u8]) -> anyhow::Result<Cid>;
    fn from_id_string(prefix: &str, cid: &str) -> anyhow::Result<Cid>;
    fn to_id_string(&self, prefix: &str) -> String;
}

impl CidExt for Cid {
    fn ensure(&self, input: &[u8]) -> anyhow::Result<()> {
        match self.hash().code() {
            SHA2_256_CODE => {
                let input_digest = sha256_hash(input)?;
                if self.hash().digest() != input_digest.as_slice() {
                    anyhow::bail!(
                        "Invalid cid {:?} != {:?} payload: {:?}",
                        input_digest.as_slice(),
                        self.hash().digest(),
                        input
                    );
                }
            }
            _ => anyhow::bail!("Invalid alg"),
        }
        Ok(())
    }

    fn create(code: u64, input: &[u8]) -> anyhow::Result<Self> {
        let input_digest = sha256_hash(input)?;
        let mh = Multihash::<64>::wrap(SHA2_256_CODE, &input_digest).map_err(anyhow::Error::msg)?;
        Ok(Cid::new_v1(code, mh))
    }
    
    fn from_id_string(prefix: &str, id: &str) -> anyhow::Result<Cid> {
        let prefix = format!("/idp2p/{}/", prefix);
        if let Some(cid) = id.strip_prefix(&prefix) {
            return Ok(Cid::try_from(cid).map_err(anyhow::Error::msg)?);
        }
        anyhow::bail!("Invalid id")
    }

    fn to_id_string(&self, prefix: &str) -> String {
        format!("/idp2p/{prefix}/{}", self.to_string_of_base(Base::Base32Lower).unwrap())
    }
}
