use anyhow::{bail, Result};
use cid::Cid;
use idp2p_common::ED_CODE;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdSigner {
    pub id: Cid,
    pub value: u8,
}

impl Default for IdSigner {
    fn default() -> Self {
        Self {
            id: Cid::default(),
            value: 1,
        }
    }
}

impl IdSigner {
    pub fn validate(&self) -> Result<()> {
        if self.value == 0 {
            bail!("The order of the signer must be greater than 0.");
        }
        match self.id.codec() {
            ED_CODE => {}
            _ => bail!("invalid codec"),
        }
        Ok(())
    }
}
