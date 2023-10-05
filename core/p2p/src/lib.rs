pub mod message;
pub mod swarm;
pub mod block;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DigestId;
