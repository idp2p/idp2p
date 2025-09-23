use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::types::IdClaimValue;

#[serde_as]
#[derive(Debug, Clone, Hash, Serialize, Deserialize, Eq, PartialEq)]
pub struct IdClaimCreateEvent {
    pub key: String,
    pub id: String,
    pub payload: Option<Vec<u8>>,
}

#[serde_as]
#[derive(Debug, Clone, Hash, Serialize, Deserialize, Eq, PartialEq)]
pub struct IdClaimRevokeEvent {
    pub key: String,
    pub id: String,
}

impl IdClaimCreateEvent {
    pub fn to_state(&self, valid_from: &str) -> IdClaimValue {
        IdClaimValue {
            id: self.id.to_string(),
            payload: self.payload.clone(),
            valid_from: valid_from.to_owned(),
            valid_until: None,
        }
    }

    fn get_unique_id(&self) -> String {
        format!("{}/{}", self.key, self.id)
    }
}

impl Ord for IdClaimCreateEvent {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.get_unique_id().cmp(&other.get_unique_id())
    }
}

impl PartialOrd for IdClaimCreateEvent {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl IdClaimRevokeEvent {
    fn get_unique_id(&self) -> String {
        format!("{}/{}", self.key, self.id)
    }
}

impl Ord for IdClaimRevokeEvent {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.get_unique_id().cmp(&other.get_unique_id())
    }
}

impl PartialOrd for IdClaimRevokeEvent {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
