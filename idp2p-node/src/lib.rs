use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum IdentityEvent {
    Created { id: String },
    Updated { id: String },
    Published { id: String },
    ReceivedJwm { id: String, jwm: String },
}

pub mod message;
pub mod node;
pub mod store;
