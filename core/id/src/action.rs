use anyhow::Result;
use cid::Cid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdActionKind {
    AddMediator(String),
    RemoveMediator(String),
    UpdateState(Cid),
}

impl IdActionKind {
    pub fn validate(&self) -> Result<()> {
        match self {
            _ => {}
        }
        Ok(())
    }
}
