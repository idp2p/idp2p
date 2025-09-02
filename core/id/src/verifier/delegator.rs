use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct IdDelegator {
    pub id: String,    
    pub scope: Vec<String>,
}

impl IdDelegator {
    pub fn to_state(&self, valid_from: &str) -> crate::types::IdDelegator {
        crate::types::IdDelegator {
            id: self.id.to_owned(),
            scope: self.scope.to_owned(),
            valid_from: valid_from.to_owned(),
            valid_until: None,
        }
    }
}

impl Eq for IdDelegator {}

impl PartialEq for IdDelegator {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Ord for IdDelegator {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for IdDelegator {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

