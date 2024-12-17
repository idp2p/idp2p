use std::str::FromStr;

use cid::Cid;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum IdTopic {
    Id(Cid),
    Other(String)
} 

impl FromStr for IdTopic {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> { 
        todo!()
    }
}