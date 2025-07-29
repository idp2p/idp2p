use idp2p_common::bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

mod state;
mod inception;

/// Represents the kind of key required for an event.
///
/// This enum defines the different types of keys that can be used for event validation.
/// Each variant corresponds to a specific type of key, such as the current key, the next key, or a delegation key.
///
/// Example:
/// ```
/// let kind = IdKeyKind::CurrentKey;
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "id")]
pub enum IdKeyRule {
    #[serde(rename = "current-key")]
    CurrentKey,
    #[serde(rename = "next-key")]
    NextKey,
    #[serde(rename = "delegation-key")]
    DelegationKey(String),
}

/// Event rule item
///
/// Represents an item in an event rule.
///
/// Each item specifies the kind of key required and the threshold for that kind.
///
/// Example:
/// ```
/// let item = IdEventRuleItem { kind: IdKeyRule::CurrentKey, threshold: 1 };
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdEventRuleItem {
    pub key_rule: IdKeyRule,
    pub threshold: u8,
}

/// Event rule
///
/// Represents a rule for event validation.
///
/// Each rule is a array of items which specify authorization requirements.
/// At least one item in the vector must be satisfied in order to add an event.
/// Each authorization requirement is a vector of EventRuleItem which specifies the kind of key required and the threshold for that kind.
/// All items in the vector must be satisfied.
///
/// Example:
/// ```
/// let rule = vec![
///     vec![EventRuleItem { kind: IdKeyKind::CurrentKey, threshold: 1 }, EventRuleItem { kind: IdKeyKind::NextKey, threshold: 2 }],
/// ];
/// ```
pub type IdEventRule = Vec<Vec<IdEventRuleItem>>;

/// Signer
///
/// Represents a signer of an identifier.
#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdSigner {
    /// Public key of the signer.
    #[serde_as(as = "Bytes")]
    pub public_key: Vec<u8>,
    /// Valid from timestamp.
    pub valid_from: i64,
    /// Valid to timestamp.
    pub valid_to: Option<i64>,
}

impl IdSigner {
    pub fn new(public_key: &[u8]) -> Self {
        Self {
            public_key: public_key.to_vec(),
            valid_from: chrono::Utc::now().timestamp(),
            valid_to: None,
        }
    }
    pub fn is_valid(&self, now: i64) -> bool {
        self.valid_from <= now && (self.valid_to.is_none() || self.valid_to.unwrap() >= now)
    }
}
