use crate::IdConfig;
use anyhow::{bail, Result};

impl Default for IdConfig {
    fn default() -> Self {
        Self {
            action_quorum: 1,
            config_quorum: 2,
            cancel_quorum: 2,
            key_reuse: true,
        }
    }
}

impl IdConfig {
    pub fn validate(&self) -> Result<()> {
        if self.action_quorum <= 0 {
            bail!("The quorum must be greater than 0.");
        }
        if self.config_quorum <= 0 {
            bail!("The quorum must be greater than 0.");
        }
        if self.cancel_quorum <= 0 {
            bail!("The quorum must be greater than 0.");
        }
        Ok(())
    }
}   