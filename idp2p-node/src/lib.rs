use idp2p_common::{thiserror::Error};
use serde::{Deserialize, Serialize};

#[derive(Error, Debug)]
pub enum IdentityNodeError {
    #[error("Unknown")]
    Unknown,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum IdentityStoreEvent {
    Created { id: String },
    PostHandled { id: String },
    GetHandled { id: String },
    JwmCreated { jwm: String },
    JwmReceived { jwm: String },
}

pub mod behaviour;
pub mod builder;
pub mod id_store;

