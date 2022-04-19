use idp2p_common::{thiserror::Error};

#[derive(Error, Debug)]
pub enum IdentityNodeError {
    #[error("Unknown")]
    Unknown,
}

pub mod store;
pub mod gossip;
pub mod swarm;
pub mod req_res;

