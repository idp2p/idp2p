use serde::{Deserialize, Serialize};
use idp2p_common::bytes::Bytes;
use serde_with::serde_as;

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