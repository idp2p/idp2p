use alloc::collections::BTreeSet;

use crate::internal::signer::IdSigner;
use alloc::string::String;
use cid::Cid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum IdEventKind {
    /// Should be signed with state.current_signers
    /// The total number of signers should be greater than or equal the state.threshold
    Interaction {
        merkle_proof: String
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
        next_id_proof: String,
    },
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdEvent {
    /// Event number
    pub sn: u64,
    
    /// Event version
    pub version: String,

    /// Event version patch
    pub patch: Cid,

    /// Timestamp of event
    pub timestamp: i64,

    /// Previous event id
    pub previous: String,

    /// Event body
    pub body: IdEventKind,
}
