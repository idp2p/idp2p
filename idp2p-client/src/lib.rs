use std::collections::HashMap;

use idp2p_core::store::IdEntry;
use serde::{Deserialize, Serialize};
use idp2p_common::{encode_vec, anyhow::Result};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct IdConfig {
    #[serde(with = "encode_vec")]
    pub secret: Vec<u8>,
    pub listen_ip: String,
    pub listen_port: u16,
    pub identities: HashMap<String, IdEntry>,
    pub remote_addr: Option<String>,
}

pub trait IdConfigResolver {
    fn get_config(&self, ip: &str, port: u16, remote: Option<String>) ->  Result<IdConfig>;
}


pub mod behaviour;
pub mod builder;
pub mod commands;
pub mod file_db;
pub mod swarm;