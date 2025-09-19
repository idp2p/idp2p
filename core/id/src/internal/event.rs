use alloc::collections::BTreeSet;

use crate::internal::{
    claim::{IdClaimCreateEvent, IdClaimRevokeEvent},
    signer::IdSigner,
};
use alloc::string::String;
use cid::Cid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum IdEventKind {
    /// Should be signed with state.current_signers
    /// The total number of signers should be greater than or equal the state.threshold
    Interaction {
        new_claims: BTreeSet<IdClaimCreateEvent>,
        revoked_claims: BTreeSet<IdClaimRevokeEvent>,
    },

    /// Should be signed with signers and new_signers
    /// The total number of signers + new_signers should be greater than or equal the current threshold
    /// The total number of signers should be greater than or equal state.next_threshold
    /// The total number of next_signers should be equal next_threshold
    /// All signers should be in the state.next_signers
    Rotation {
        /// new threshold
        threshold: u8,
        /// new next_threshold
        next_threshold: u8,
        /// signers in state.next_signers
        revealed_signers: BTreeSet<IdSigner>,
        // if the threshold increases
        new_signers: BTreeSet<IdSigner>,
        /// next signer ids
        next_signers: BTreeSet<String>,
    },

    /// Should be signed with signers
    /// All signers should be in the state.next_signers
    /// The total number of signers should be greater than or equal state.next_threshold
    Revocation {
        revealed_signers: BTreeSet<IdSigner>,
    },
    /// Should be signed with signers
    /// All signers should be in the state.next_signers
    /// The total number of signers should be greater than or equal state.next_threshold
    Migration {
        revealed_signers: BTreeSet<IdSigner>,
        next_id: String,
    },
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdEvent {
    pub version: String,
    /// Timestamp of event
    pub timestamp: i64,

    // The compoenent
    pub component: Cid,

    /// Previous event id
    pub previous: String,

    /// Event body
    pub body: IdEventKind,
}
