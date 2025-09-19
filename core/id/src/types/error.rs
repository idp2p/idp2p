use crate::internal::error::IdEventError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct Idp2pError {
    pub code: String,
    pub message: String,
}

impl From<IdEventError> for Idp2pError {
    fn from(e: IdEventError) -> Self {
        let code = e.as_ref().to_lowercase();
        let message = e.to_string();
        Idp2pError { code, message }
    }
}