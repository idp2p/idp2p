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
        if s.starts_with("id:") {
            Ok(IdTopic::Id(Cid::from_str(&s[3..])?))
        } else if s.starts_with("o:") {
            Ok(IdTopic::Other(s[2..].to_string()))
        } else {
            anyhow::bail!("Invalid topic");
        }
    }
}