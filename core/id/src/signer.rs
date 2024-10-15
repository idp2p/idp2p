use crate::IdSigner;
use anyhow::{bail, Result};
use cid::Cid;
use idp2p_common::{cid::CidExt, ED_CODE};

impl Default for IdSigner {
    fn default() -> Self {
        Self {
            id: vec![0;32],
            value: 1,
        }
    }
}

impl IdSigner {
    pub fn validate(&self) -> Result<()> {
        let cid = Cid::from_bytes(&self.id)?;
        if self.value == 0 {
            bail!("The order of the signer must be greater than 0.");
        }
        match cid.codec() {
            ED_CODE => {}
            _ => bail!("invalid codec"),
        }
        Ok(())
    }
}