use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::types::IdClaimValue;


#[serde_as]
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct IdClaim {
    pub kind: String,
    pub id: String,
    pub value: IdClaimValue
}

impl IdClaim {
    pub fn to_state(&self, valid_from: &str) -> crate::types::IdClaim {
        crate::types::IdClaim {
            kind: self.kind.to_owned(),
            id: self.id.to_owned(),
            value: self.value.to_owned(),
            valid_from: valid_from.to_owned(),
            valid_until: None,
        }
    }
}

impl Eq for IdClaim {}

impl PartialEq for IdClaim {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Ord for IdClaim {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for IdClaim {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}