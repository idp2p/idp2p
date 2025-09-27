use alloc::collections::BTreeSet;
use alloc::str::FromStr;
use chrono::{DateTime, Utc};
use cid::Cid;
use idp2p_common::{
    CBOR_CODE, ED_CODE, bytes::Bytes, cid::CidExt, error::CommonError, verification::ed25519,
};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::{
    VALID_FROM, VERSION,
    host::verify_proof,
    internal::{
        error::IdEventError, event::IdEvent, inception::IdInception, signer::IdSigner,
        utils::Timestamp,
    },
    types::{IdProof, IdState},
};

macro_rules! ensure {
    ($cond:expr, $error:expr) => {
        if !($cond) {
            return Err($error);
        }
    };
}

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdEventReceipt {
    pub id: String,
    pub version: String,
    pub created_at: String,
    #[serde_as(as = "Bytes")]
    pub payload: Vec<u8>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub proofs: Vec<IdProof>,
}

impl Ord for IdEventReceipt {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for IdEventReceipt {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl IdEventReceipt {
    fn verify_proofs(&self, signers: &BTreeSet<IdSigner>) -> Result<(), IdEventError> {
        for proof in self.proofs.iter() {
            let mut seen: BTreeSet<String> = BTreeSet::new();
            if !seen.insert(proof.id.clone()) {
                return Err(IdEventError::invalid_proof(
                    &proof.key_id,
                    "duplicate proof",
                ));
            }
            if proof.purpose != "id-delegation" {
                crate::host::verify_proof(&proof, &self.payload)
                    .map_err(|e| IdEventError::invalid_proof(&proof.key_id, &e.code))?;
            } else {
                ensure!(
                    proof.purpose == "id-inception",
                    IdEventError::InvalidPayload
                );
                proof.verify(&self.payload, &signers)?;
            }
        }
        Ok(())
    }

    pub fn verify_inception(&self) -> Result<IdState, IdEventError> {
        ensure!(self.version == VERSION, IdEventError::UnsupportedVersion);
        let id = Cid::from_str(&self.id)?;
        id.ensure(&self.payload, vec![CBOR_CODE])?;
        let inception: IdInception = idp2p_common::cbor::decode(&self.payload)
            .map_err(|e| CommonError::DecodeError(e.to_string()))?;

        let valid_from: DateTime<Utc> = VALID_FROM
            .parse()
            .map_err(|_| IdEventError::InvalidTimestamp)?;
        let total_signers = inception.signers.len() as u8;
        let total_next_signers = inception.next_signers.len() as u8;
        let total_signatures = self.proofs.len() as u8;

        // Compare seconds to seconds
        ensure!(
            inception.timestamp > valid_from.timestamp(),
            IdEventError::InvalidTimestamp
        );

        ensure!(
            total_signers >= total_signatures,
            IdEventError::LackOfMinProofs
        );

        ensure!(
            total_signatures >= inception.threshold,
            IdEventError::LackOfMinProofs
        );
        ensure!(
            inception.version == VERSION,
            IdEventError::UnsupportedVersion
        );
        ensure!(inception.threshold >= 1, IdEventError::ThresholdNotMatch);

        ensure!(
            total_next_signers >= inception.next_threshold,
            IdEventError::NextThresholdNotMatch
        );

        // Validate next signer ids
        for next_kid_str in &inception.next_signers {
            let next_kid = Cid::from_str(next_kid_str)?;
            ensure!(
                next_kid.codec() == ED_CODE,
                IdEventError::InvalidNextSigner(next_kid_str.clone())
            );
        }

        let timestamp: String = String::try_from(Timestamp(inception.timestamp))?;
        self.verify_proofs(&inception.signers)?;
       
        let mut id_state = IdState {
            id: self.id.clone(),
            event_id: self.id.clone(),
            event_timestamp: timestamp.clone(),
            prior_id: inception.prior_id.clone(),
            next_id: None,
            threshold: inception.threshold,
            next_threshold: inception.next_threshold,
            signers: inception
                .signers
                .clone()
                .into_iter()
                .map(|s| s.to_state(&timestamp))
                .collect(),
            current_signers: inception
                .signers
                .into_iter()
                .map(|signer| signer.id)
                .collect(),
            next_signers: inception.next_signers.into_iter().collect(),
            delegators: vec![],
            claims: vec![],
            revoked: false,
            revoked_at: None,
        };
        for event in inception.claims {
            id_state.add_claim(event, &timestamp);
        }
        Ok(id_state)
    }

    pub fn verify_event(&self, state: &mut IdState) -> Result<IdState, IdEventError> {
        let mut state = state.to_owned();
        let cid = Cid::from_str(&self.id)?;
        cid.ensure(&self.payload, vec![CBOR_CODE])?;
        let event: IdEvent = idp2p_common::cbor::decode(&self.payload)?;

        ensure!(event.version == VERSION, IdEventError::UnsupportedVersion);

        // Timestamp check (seconds)
        let valid_from: DateTime<Utc> = VALID_FROM
            .parse()
            .map_err(|_| IdEventError::InvalidTimestamp)?;
        ensure!(
            event.timestamp >= valid_from.timestamp(),
            IdEventError::InvalidTimestamp
        );
        // Previous event check
        ensure!(
            event.previous == state.event_id,
            IdEventError::PreviousNotMatch
        );

        let timestamp: String = String::try_from(Timestamp(event.timestamp))?;
        use crate::internal::event::IdEventKind::*;
        match event.body {
            Interaction {
                new_claims,
                revoked_claims,
            } => {
                let proof_signers: BTreeSet<IdSigner> = state
                    .signers
                    .iter()
                    .map(|s| IdSigner {
                        id: s.id.clone(),
                        public_key: s.public_key.clone(),
                    })
                    .collect();
                // Require at least `state.threshold` proofs
                ensure!(
                    self.proofs.len() as u8 >= state.threshold,
                    IdEventError::LackOfMinProofs
                );
                self.verify_proofs(&proof_signers)?;
                for event in new_claims {
                    state.add_claim(event, &timestamp);
                }
                for event in revoked_claims {
                    state.revoke_claim(event, &timestamp)?;
                }
            }
            Rotation {
                threshold,
                next_threshold,
                revealed_signers,
                new_signers,
                next_signers,
            } => {
                let all_signers: BTreeSet<IdSigner> =
                    revealed_signers.union(&new_signers).cloned().collect();

                let total_signers = all_signers.len() as u8;
                let total_revealed_signers = revealed_signers.len() as u8;
                let total_next_signers = next_signers.len() as u8;
                ensure!(
                    total_signers == self.proofs.len() as u8,
                    IdEventError::ThresholdNotMatch
                );
                ensure!(total_signers >= threshold, IdEventError::ThresholdNotMatch);

                ensure!(
                    total_revealed_signers >= state.next_threshold,
                    IdEventError::ThresholdNotMatch
                );
                for signer in revealed_signers {
                    ensure!(
                        state.next_signers.iter().any(|s| s == &signer.id),
                        IdEventError::ThresholdNotMatch
                    );
                }

                ensure!(
                    total_next_signers >= next_threshold,
                    IdEventError::NextThresholdNotMatch
                );
                for next_kid_str in &next_signers {
                    let next_kid = Cid::from_str(next_kid_str)?;
                    ensure!(
                        next_kid.codec() == ED_CODE,
                        IdEventError::InvalidNextSigner(next_kid_str.clone())
                    );
                }
                self.verify_proofs(&all_signers)?;
                for signer_id in &state.current_signers {
                    let signer = state
                        .signers
                        .iter_mut()
                        .find(|s| &s.id == signer_id)
                        .ok_or(IdEventError::InvalidSigner(signer_id.clone()))?;
                    signer.valid_until = Some(timestamp.clone());
                }
                let all_signers: Vec<crate::types::IdSigner> = all_signers
                    .clone()
                    .into_iter()
                    .map(|s| s.to_state(&timestamp))
                    .collect();
                state.signers.extend(all_signers);
                state.next_signers = next_signers.into_iter().collect();
                state.threshold = threshold;
                state.next_threshold = next_threshold;
            }
            Revocation { revealed_signers } => {
                ensure!(
                    revealed_signers.len() == self.proofs.len(),
                    IdEventError::ThresholdNotMatch
                );

                ensure!(
                    revealed_signers.len() as u8 >= state.next_threshold,
                    IdEventError::ThresholdNotMatch
                );
                for signer in &revealed_signers {
                    ensure!(
                        state.next_signers.iter().any(|s| s == &signer.id),
                        IdEventError::ThresholdNotMatch
                    );
                }
                self.verify_proofs(&revealed_signers)?;
                state.next_signers = vec![];
                state.revoked = true;
                state.revoked_at = Some(timestamp.clone());
            }
            Migration {
                revealed_signers,
                next_id,
            } => {
                ensure!(
                    revealed_signers.len() == self.proofs.len(),
                    IdEventError::ThresholdNotMatch
                );

                ensure!(
                    revealed_signers.len() as u8 >= state.next_threshold,
                    IdEventError::ThresholdNotMatch
                );
                for signer in &revealed_signers {
                    ensure!(
                        state.next_signers.iter().any(|s| s == &signer.id),
                        IdEventError::ThresholdNotMatch
                    );
                }
                self.verify_proofs(&revealed_signers)?;
                state.next_signers = vec![];
                state.next_id = Some(next_id);
            }
        }

        // Update event timestamp in state
        state.event_timestamp = timestamp.clone();
        state.event_id = self.id.clone();

        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal::event::IdEventKind::*;
    use crate::internal::signer::IdSigner as InternalSigner;
    use chrono::Utc;
    use ed25519_dalek::Signer as _;
    use ed25519_dalek::{SigningKey, VerifyingKey};
    use idp2p_common::{CBOR_CODE, ED_CODE, cbor};
    use rand::rngs::OsRng;

    fn create_signer() -> (String, VerifyingKey, SigningKey) {
        let mut csprng = OsRng;
        let signing_key: SigningKey = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        let id = Cid::create(ED_CODE, verifying_key.as_bytes())
            .unwrap()
            .to_string();
        (id, verifying_key, signing_key)
    }

    fn sign_receipt(payload: &[u8], creator: &str, kid: &str, sk: &SigningKey) -> IdProof {
        let signature = sk.sign(payload);
        let id = Cid::create(CBOR_CODE, payload).unwrap().to_string();
        IdProof {
            id: id,
            did: creator.into(),
            key_id: kid.into(),
            created: Utc::now().to_rfc3339(),
            purpose: "id-inception".into(),
            signature: signature.to_vec(),
        }
    }

    fn base_state_with_signer(id: &str, pubkey: &[u8]) -> IdState {
        IdState {
            id: "test".into(),
            event_id: "prev".into(),
            event_timestamp: Utc::now().to_rfc3339(),
            prior_id: None,
            next_id: None,
            threshold: 1,
            next_threshold: 1,
            signers: vec![crate::types::IdSigner {
                id: id.to_string(),
                public_key: pubkey.to_vec(),
                valid_from: Utc::now().to_rfc3339(),
                valid_until: None,
            }],
            current_signers: vec![id.to_string()],
            next_signers: vec![id.to_string()],
            delegators: vec![],
            claims: vec![],
            revoked: false,
            revoked_at: None,
        }
    }

    #[test]
    fn test_interaction_event_success() {
        let (sid, vk, sk) = create_signer();
        let mut state = base_state_with_signer(&sid, vk.as_bytes());

        let event = IdEvent {
            version: VERSION.into(),
            timestamp: Utc::now().timestamp(),
            component: Cid::default(),
            previous: state.event_id.clone(),
            body: Interaction {
                new_claims: BTreeSet::new(),
                revoked_claims: BTreeSet::new(),
            },
        };
        let payload = cbor::encode(&event);
        let receipt = IdEventReceipt {
            id: Cid::create(CBOR_CODE, &payload).unwrap().to_string(),
            version: VERSION.into(),
            created_at: Utc::now().to_rfc3339(),
            payload: payload.clone(),
            proofs: vec![sign_receipt(&payload, &state.id, &sid, &sk)],
        };

        let updated = receipt
            .verify_event(&mut state)
            .expect("interaction should pass");
        assert_eq!(updated.event_id, receipt.id);
    }

    #[test]
    fn test_event_updates_event_timestamp() {
        let (sid, vk, sk) = create_signer();
        let mut state = base_state_with_signer(&sid, vk.as_bytes());
        let ts = Utc::now().timestamp();
        let expected_ts: String = String::try_from(crate::internal::utils::Timestamp(ts)).unwrap();

        let event = IdEvent {
            version: VERSION.into(),
            timestamp: ts,
            component: Cid::default(),
            previous: state.event_id.clone(),
            body: Interaction {
                new_claims: BTreeSet::new(),
                revoked_claims: BTreeSet::new(),
            },
        };
        let payload = cbor::encode(&event);
        let receipt = IdEventReceipt {
            id: Cid::create(CBOR_CODE, &payload).unwrap().to_string(),
            version: VERSION.into(),
            created_at: Utc::now().to_rfc3339(),
            payload: payload.clone(),
            proofs: vec![sign_receipt(&payload, &state.id, &sid, &sk)],
        };

        let updated = receipt
            .verify_event(&mut state)
            .expect("event verification should pass");
        assert_eq!(updated.event_timestamp, expected_ts);
    }

    #[test]
    fn test_invalid_version() {
        let (sid, vk, sk) = create_signer();
        let mut state = base_state_with_signer(&sid, vk.as_bytes());

        let event = IdEvent {
            version: "0.9".into(),
            timestamp: Utc::now().timestamp(),
            component: Cid::default(),
            previous: state.event_id.clone(),
            body: Interaction {
                new_claims: BTreeSet::new(),
                revoked_claims: BTreeSet::new(),
            },
        };
        let payload = cbor::encode(&event);
        let receipt = IdEventReceipt {
            id: Cid::create(CBOR_CODE, &payload).unwrap().to_string(),
            version: VERSION.into(),
            created_at: Utc::now().to_rfc3339(),
            payload: payload.clone(),
            proofs: vec![sign_receipt(&payload, &state.id, &sid, &sk)],
        };
        let err = receipt.verify_event(&mut state).unwrap_err();
        assert!(matches!(err, IdEventError::UnsupportedVersion));
    }

    #[test]
    fn test_invalid_timestamp() {
        let (sid, vk, sk) = create_signer();
        let mut state = base_state_with_signer(&sid, vk.as_bytes());
        let event = IdEvent {
            version: VERSION.into(),
            timestamp: 0,
            component: Cid::default(),
            previous: state.event_id.clone(),
            body: Interaction {
                new_claims: BTreeSet::new(),
                revoked_claims: BTreeSet::new(),
            },
        };
        let payload = cbor::encode(&event);
        let receipt = IdEventReceipt {
            id: Cid::create(CBOR_CODE, &payload).unwrap().to_string(),
            version: VERSION.into(),
            created_at: Utc::now().to_rfc3339(),
            payload: payload.clone(),
            proofs: vec![sign_receipt(&payload, &state.id, &sid, &sk)],
        };
        let err = receipt.verify_event(&mut state).unwrap_err();
        assert!(matches!(err, IdEventError::InvalidTimestamp));
    }

    #[test]
    fn test_interaction_insufficient_proofs_for_threshold() {
        let (sid1, vk1, sk1) = create_signer();
        let (sid2, vk2, _sk2) = create_signer();
        let mut state = base_state_with_signer(&sid1, vk1.as_bytes());
        state.signers.push(crate::types::IdSigner {
            id: sid2.clone(),
            public_key: vk2.as_bytes().to_vec(),
            valid_from: Utc::now().to_rfc3339(),
            valid_until: None,
        });
        state.current_signers = vec![sid1.clone(), sid2.clone()];
        state.threshold = 2;

        let event = IdEvent {
            version: VERSION.into(),
            timestamp: Utc::now().timestamp(),
            component: Cid::default(),
            previous: state.event_id.clone(),
            body: Interaction {
                new_claims: BTreeSet::new(),
                revoked_claims: BTreeSet::new(),
            },
        };
        let payload = cbor::encode(&event);
        let receipt = IdEventReceipt {
            id: Cid::create(CBOR_CODE, &payload).unwrap().to_string(),
            version: VERSION.into(),
            created_at: Utc::now().to_rfc3339(),
            payload: payload.clone(),
            proofs: vec![sign_receipt(&payload, &state.id, &sid1, &sk1)],
        };
        let err = receipt.verify_event(&mut state).unwrap_err();
        assert!(matches!(err, IdEventError::LackOfMinProofs));
    }

    #[test]
    fn test_previous_mismatch() {
        let (sid, vk, sk) = create_signer();
        let mut state = base_state_with_signer(&sid, vk.as_bytes());
        let event = IdEvent {
            version: VERSION.into(),
            timestamp: Utc::now().timestamp(),
            component: Cid::default(),
            previous: "some-other".into(),
            body: Interaction {
                new_claims: BTreeSet::new(),
                revoked_claims: BTreeSet::new(),
            },
        };
        let payload = cbor::encode(&event);
        let receipt = IdEventReceipt {
            id: Cid::create(CBOR_CODE, &payload).unwrap().to_string(),
            version: VERSION.into(),
            created_at: Utc::now().to_rfc3339(),
            payload: payload.clone(),
            proofs: vec![sign_receipt(&payload, &state.id, &sid, &sk)],
        };
        let err = receipt.verify_event(&mut state).unwrap_err();
        assert!(matches!(err, IdEventError::PreviousNotMatch));
    }

    #[test]
    fn test_rotation_updates_thresholds_and_next_signers() {
        let (sid, vk, sk) = create_signer();
        let mut state = base_state_with_signer(&sid, vk.as_bytes());

        let mut revealed = BTreeSet::new();
        revealed.insert(InternalSigner {
            id: sid.clone(),
            public_key: vk.as_bytes().to_vec(),
        });
        let new_next_signers: BTreeSet<String> = [sid.clone()].into_iter().collect();
        let event = IdEvent {
            version: VERSION.into(),
            timestamp: Utc::now().timestamp(),
            component: Cid::default(),
            previous: state.event_id.clone(),
            body: Rotation {
                threshold: 1,
                next_threshold: 1,
                revealed_signers: revealed,
                new_signers: BTreeSet::new(),
                next_signers: new_next_signers,
            },
        };
        let payload = cbor::encode(&event);
        let receipt = IdEventReceipt {
            id: Cid::create(CBOR_CODE, &payload).unwrap().to_string(),
            version: VERSION.into(),
            created_at: Utc::now().to_rfc3339(),
            payload: payload.clone(),
            proofs: vec![sign_receipt(&payload, &state.id, &sid, &sk)],
        };

        let updated = receipt
            .verify_event(&mut state)
            .expect("rotation should pass");
        assert_eq!(updated.threshold, 1);
        assert_eq!(updated.next_threshold, 1);
        assert_eq!(updated.next_signers, vec![sid]);
        assert_eq!(updated.event_id, receipt.id);
    }

    #[test]
    fn test_rotation_signers_must_match_proofs_len() {
        let (sid1, vk1, sk1) = create_signer();
        let (sid2, vk2, _sk2) = create_signer();
        let mut state = base_state_with_signer(&sid1, vk1.as_bytes());
        state.next_signers = vec![sid1.clone(), sid2.clone()];
        state.next_threshold = 1;
        let mut revealed = BTreeSet::new();
        revealed.insert(InternalSigner {
            id: sid1.clone(),
            public_key: vk1.as_bytes().to_vec(),
        });
        revealed.insert(InternalSigner {
            id: sid2.clone(),
            public_key: vk2.as_bytes().to_vec(),
        });
        let mut next_signers: BTreeSet<String> = BTreeSet::new();
        next_signers.insert(sid1.clone());
        let event = IdEvent {
            version: VERSION.into(),
            timestamp: Utc::now().timestamp(),
            component: Cid::default(),
            previous: state.event_id.clone(),
            body: Rotation {
                threshold: 1,
                next_threshold: 1,
                revealed_signers: revealed,
                new_signers: BTreeSet::new(),
                next_signers,
            },
        };
        let payload = cbor::encode(&event);
        let receipt = IdEventReceipt {
            id: Cid::create(CBOR_CODE, &payload).unwrap().to_string(),
            version: VERSION.into(),
            created_at: Utc::now().to_rfc3339(),
            payload: payload.clone(),
            proofs: vec![sign_receipt(&payload, &state.id, &sid1, &sk1)],
        };
        let err = receipt.verify_event(&mut state).unwrap_err();
        assert!(matches!(err, IdEventError::ThresholdNotMatch));
    }

    #[test]
    fn test_rotation_insufficient_next_signers() {
        let (sid, vk, sk) = create_signer();
        let mut state = base_state_with_signer(&sid, vk.as_bytes());
        let mut revealed = BTreeSet::new();
        revealed.insert(InternalSigner {
            id: sid.clone(),
            public_key: vk.as_bytes().to_vec(),
        });
        let next_signers: BTreeSet<String> = [].into_iter().collect();
        let event = IdEvent {
            version: VERSION.into(),
            timestamp: Utc::now().timestamp(),
            component: Cid::default(),
            previous: state.event_id.clone(),
            body: Rotation {
                threshold: 1,
                next_threshold: 2,
                revealed_signers: revealed,
                new_signers: BTreeSet::new(),
                next_signers,
            },
        };
        let payload = cbor::encode(&event);
        let receipt = IdEventReceipt {
            id: Cid::create(CBOR_CODE, &payload).unwrap().to_string(),
            version: VERSION.into(),
            created_at: Utc::now().to_rfc3339(),
            payload: payload.clone(),
            proofs: vec![sign_receipt(&payload, &state.id, &sid, &sk)],
        };
        let err = receipt.verify_event(&mut state).unwrap_err();
        assert!(matches!(err, IdEventError::NextThresholdNotMatch));
    }

    #[test]
    fn test_revocation_event_success() {
        let (sid, vk, sk) = create_signer();
        let mut state = base_state_with_signer(&sid, vk.as_bytes());

        let mut revealed = BTreeSet::new();
        revealed.insert(InternalSigner {
            id: sid.clone(),
            public_key: vk.as_bytes().to_vec(),
        });
        let event = IdEvent {
            version: VERSION.into(),
            timestamp: Utc::now().timestamp(),
            component: Cid::default(),
            previous: state.event_id.clone(),
            body: Revocation {
                revealed_signers: revealed,
            },
        };
        let payload = cbor::encode(&event);
        let receipt = IdEventReceipt {
            id: Cid::create(CBOR_CODE, &payload).unwrap().to_string(),
            version: VERSION.into(),
            created_at: Utc::now().to_rfc3339(),
            payload: payload.clone(),
            proofs: vec![sign_receipt(&payload, &state.id, &sid, &sk)],
        };

        let updated = receipt
            .verify_event(&mut state)
            .expect("revocation should pass");
        assert!(updated.revoked);
        assert!(updated.revoked_at.is_some());
        assert_eq!(updated.event_id, receipt.id);
    }

    #[test]
    fn test_revocation_threshold_not_met() {
        let (sid1, vk1, sk1) = create_signer();
        let (sid2, _vk2, _sk2) = create_signer();
        let mut state = base_state_with_signer(&sid1, vk1.as_bytes());
        state.next_threshold = 2;
        state.next_signers = vec![sid1.clone(), sid2.clone()];
        let mut revealed = BTreeSet::new();
        revealed.insert(InternalSigner {
            id: sid1.clone(),
            public_key: vk1.as_bytes().to_vec(),
        });
        let event = IdEvent {
            version: VERSION.into(),
            timestamp: Utc::now().timestamp(),
            component: Cid::default(),
            previous: state.event_id.clone(),
            body: Revocation {
                revealed_signers: revealed,
            },
        };
        let payload = cbor::encode(&event);
        let receipt = IdEventReceipt {
            id: Cid::create(CBOR_CODE, &payload).unwrap().to_string(),
            version: VERSION.into(),
            created_at: Utc::now().to_rfc3339(),
            payload: payload.clone(),
            proofs: vec![sign_receipt(&payload, &state.id, &sid1, &sk1)],
        };
        let err = receipt.verify_event(&mut state).unwrap_err();
        assert!(matches!(err, IdEventError::ThresholdNotMatch));
    }

    #[test]
    fn test_migration_event_sets_next_id() {
        let (sid, vk, sk) = create_signer();
        let mut state = base_state_with_signer(&sid, vk.as_bytes());

        let mut revealed = BTreeSet::new();
        revealed.insert(InternalSigner {
            id: sid.clone(),
            public_key: vk.as_bytes().to_vec(),
        });
        let event = IdEvent {
            version: VERSION.into(),
            timestamp: Utc::now().timestamp(),
            component: Cid::default(),
            previous: state.event_id.clone(),
            body: Migration {
                revealed_signers: revealed,
                next_id: "did:idp2p:new".into(),
            },
        };
        let payload = cbor::encode(&event);
        let receipt = IdEventReceipt {
            id: Cid::create(CBOR_CODE, &payload).unwrap().to_string(),
            version: VERSION.into(),
            created_at: Utc::now().to_rfc3339(),
            payload: payload.clone(),
            proofs: vec![sign_receipt(&payload, &state.id, &sid, &sk)],
        };

        let updated = receipt
            .verify_event(&mut state)
            .expect("migration should pass");
        assert_eq!(updated.next_id, Some("did:idp2p:new".into()));
        assert_eq!(updated.event_id, receipt.id);
    }

    #[test]
    fn test_rotation_invalid_next_signer_codec() {
        let (sid, vk, sk) = create_signer();
        let mut state = base_state_with_signer(&sid, vk.as_bytes());
        let mut revealed = BTreeSet::new();
        revealed.insert(InternalSigner {
            id: sid.clone(),
            public_key: vk.as_bytes().to_vec(),
        });
        let invalid_next = Cid::create(CBOR_CODE, vk.as_bytes()).unwrap().to_string();
        let next_signers: BTreeSet<String> = [invalid_next.clone()].into_iter().collect();
        let event = IdEvent {
            version: VERSION.into(),
            timestamp: Utc::now().timestamp(),
            component: Cid::default(),
            previous: state.event_id.clone(),
            body: Rotation {
                threshold: 1,
                next_threshold: 1,
                revealed_signers: revealed,
                new_signers: BTreeSet::new(),
                next_signers,
            },
        };
        let payload = cbor::encode(&event);
        let receipt = IdEventReceipt {
            id: Cid::create(CBOR_CODE, &payload).unwrap().to_string(),
            version: VERSION.into(),
            created_at: Utc::now().to_rfc3339(),
            payload: payload.clone(),
            proofs: vec![sign_receipt(&payload, &state.id, &sid, &sk)],
        };
        let err = receipt.verify_event(&mut state).unwrap_err();
        assert!(matches!(err, IdEventError::InvalidNextSigner(_)));
    }

    #[test]
    fn test_duplicate_proofs_rejected() {
        let (sid, vk, sk) = create_signer();
        let mut state = base_state_with_signer(&sid, vk.as_bytes());
        let event = IdEvent {
            version: VERSION.into(),
            timestamp: Utc::now().timestamp(),
            component: Cid::default(),
            previous: state.event_id.clone(),
            body: Interaction {
                new_claims: BTreeSet::new(),
                revoked_claims: BTreeSet::new(),
            },
        };
        let payload = cbor::encode(&event);
        let p1 = sign_receipt(&payload, &state.id, &sid, &sk);
        let p2 = sign_receipt(&payload, &state.id, &sid, &sk);
        let receipt = IdEventReceipt {
            id: Cid::create(CBOR_CODE, &payload).unwrap().to_string(),
            version: VERSION.into(),
            created_at: Utc::now().to_rfc3339(),
            payload: payload.clone(),
            proofs: vec![p1, p2],
        };
        let err = receipt.verify_event(&mut state).unwrap_err();
        assert!(matches!(err, IdEventError::InvalidProof { .. }));
    }

    #[test]
    fn test_invalid_created_format_in_proof() {
        let (sid, vk, sk) = create_signer();
        let mut state = base_state_with_signer(&sid, vk.as_bytes());
        let event = IdEvent {
            version: VERSION.into(),
            timestamp: Utc::now().timestamp(),
            component: Cid::default(),
            previous: state.event_id.clone(),
            body: Interaction {
                new_claims: BTreeSet::new(),
                revoked_claims: BTreeSet::new(),
            },
        };
        let payload = cbor::encode(&event);
        let signature = sk.sign(&payload).to_vec();
        let id = Cid::create(CBOR_CODE, &payload).unwrap().to_string();
        let proof = IdProof {
            id: id,
            did: sid.clone(),
            key_id: sid.clone(),
            created: "not-a-date".into(),
            purpose: "id-inception".into(),
            signature,
        };
        let receipt = IdEventReceipt {
            id: Cid::create(CBOR_CODE, &payload).unwrap().to_string(),
            version: VERSION.into(),
            created_at: Utc::now().to_rfc3339(),
            payload: payload.clone(),
            proofs: vec![proof],
        };
        let err = receipt.verify_event(&mut state).unwrap_err();
        assert!(matches!(err, IdEventError::InvalidProof { .. }));
    }

    #[test]
    fn test_invalid_signature_rejected() {
        let (sid, vk, sk) = create_signer();
        let mut state = base_state_with_signer(&sid, vk.as_bytes());
        let event = IdEvent {
            version: VERSION.into(),
            timestamp: Utc::now().timestamp(),
            component: Cid::default(),
            previous: state.event_id.clone(),
            body: Interaction {
                new_claims: BTreeSet::new(),
                revoked_claims: BTreeSet::new(),
            },
        };
        let payload = cbor::encode(&event);
        let mut bad_sig = sk.sign(&payload).to_vec();
        bad_sig[0] ^= 0xFF;
        let id = Cid::create(CBOR_CODE, &payload).unwrap().to_string();
        let proof = IdProof {
            id: id,
            did: sid.clone(),
            key_id: sid.clone(),
            created: Utc::now().to_rfc3339(),
            purpose: "id-inception".into(),
            signature: bad_sig,
        };
        let receipt = IdEventReceipt {
            id: Cid::create(CBOR_CODE, &payload).unwrap().to_string(),
            version: VERSION.into(),
            created_at: Utc::now().to_rfc3339(),
            payload: payload.clone(),
            proofs: vec![proof],
        };
        let err = receipt.verify_event(&mut state).unwrap_err();
        assert!(matches!(err, IdEventError::InvalidProof { .. }));
    }

    #[test]
    fn test_unknown_signer_in_proof() {
        let (sid1, vk1, _sk1) = create_signer();
        let (sid2, _vk2, sk2) = create_signer();
        let mut state = base_state_with_signer(&sid1, vk1.as_bytes());
        let event = IdEvent {
            version: VERSION.into(),
            timestamp: Utc::now().timestamp(),
            component: Cid::default(),
            previous: state.event_id.clone(),
            body: Interaction {
                new_claims: BTreeSet::new(),
                revoked_claims: BTreeSet::new(),
            },
        };
        let payload = cbor::encode(&event);
        let proof = sign_receipt(&payload, &state.id, &sid2, &sk2);
        let receipt = IdEventReceipt {
            id: Cid::create(CBOR_CODE, &payload).unwrap().to_string(),
            version: VERSION.into(),
            created_at: Utc::now().to_rfc3339(),
            payload: payload.clone(),
            proofs: vec![proof],
        };
        let err = receipt.verify_event(&mut state).unwrap_err();
        assert!(matches!(err, IdEventError::InvalidSigner(_)));
    }
}
