use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};

use crate::{IdAssertationKey, IdClaim, IdKey};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdInception {
    pub threshold: u8,
    pub timestamp: String,
    pub signers: Vec<IdKey>,
    pub next_signers: Vec<String>,
    pub actions: Vec<IdActionKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdActionKind {
    AddMediator(String),
    RemoveMediator(String),
    AddAuthentication(IdKey),
    RemoveAuthentication(String),
    AddKeyAgreement(IdKey),
    RemoveKeyAgreement(String),
    AddAssertation(IdAssertationKey),
    RemoveAssertation(String),
    AddClaim(IdClaim),
    RemoveClaim(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdEventPayload {
    // Should be signed with current keys
    Interaction(Vec<IdActionKind>),
    // Should be signed with next keys
    Rotation {
        next_threshold: u8,
        next_signers: Vec<String>,
    },
    // Should be signed with next keys
    Delegation(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdEvent {
    pub timestamp: String,
    pub previous: String,
    pub signers: Vec<String>,
    pub payload: IdEventPayload,
}
