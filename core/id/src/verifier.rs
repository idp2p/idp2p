use serde::{Deserialize, Serialize};
use idp2p_common::bytes::Bytes;
use serde_with::serde_as;

pub mod error;
pub mod inception;
pub mod event;

#[serde_as]
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct IdDelegator {
    pub id: String,    
    pub scope: Vec<String>,
}

#[serde_as]
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct IdClaim {
    pub kind: String,
    pub id: String,
    #[serde_as(as = "Bytes")]
    pub value: Vec<u8>
}

#[serde_as]
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct IdSigner {
    pub id: String,
    /// Public key of the signer.
    #[serde_as(as = "Bytes")]
    pub public_key: Vec<u8>
}

impl IdSigner {
    pub fn to_state(&self, valid_from: &str) -> crate::types::IdSigner {
        crate::types::IdSigner {
            id: self.id.to_owned(),
            public_key: self.public_key.to_owned(),
            valid_from: valid_from.to_owned(),
            valid_until: None,
        }
    }
}

impl Eq for IdSigner {}

impl PartialEq for IdSigner {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Ord for IdSigner {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for IdSigner {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
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