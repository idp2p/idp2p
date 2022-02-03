use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Jwm {
    pub from: String,
    pub to: String,
    pub kid: String,
    pub alg: String,
}
