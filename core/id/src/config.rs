use crate::IdConfig;
use anyhow::{bail, Result};

impl Default for IdConfig {
    fn default() -> Self {
        Self {
            change_state_quorum: 1,
            change_config_quorum: 2,
            revoke_event_quorum: 2,
            key_reuse: true,
        }
    }
}

impl IdConfig {
    pub fn validate(&self) -> Result<()> {
        if self.change_state_quorum <= 0 {
            bail!("The quorum must be greater than 0.");
        }
        if self.change_config_quorum <= 0 {
            bail!("The quorum must be greater than 0.");
        }
        if self.revoke_event_quorum <= 0 {
            bail!("The quorum must be greater than 0.");
        }
        Ok(())
    }
}   