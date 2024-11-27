use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdConfig {
    pub quorum: u16,
    pub key_reuse: bool,
}

impl Default for IdConfig {
    fn default() -> Self {
        Self {
            quorum: 1,
            key_reuse: true,
        }
    }
}

impl IdConfig {
    pub fn validate(&self) -> Result<()> {
        if self.quorum <= 0 {
            bail!("The quorum must be greater than 0.");
        }
        Ok(())
    }
}   