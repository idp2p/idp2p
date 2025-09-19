use std::collections::BTreeSet;

use alloc::str::FromStr;
use chrono::{DateTime, Utc};
use ciborium::cbor;
use cid::Cid;
use idp2p_common::{CBOR_CODE, ED_CODE, bytes::Bytes, cid::CidExt, ed25519, error::CommonError};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::{
    VALID_FROM, VERSION,
    types::{
        IdState,
        proof::{IdProof, IdProofReceipt},
    },
    internal::{
        error::IdEventError, event::IdEvent, inception::IdInception, signer::IdSigner,
        utils::Timestamp,
    },
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
    // Key means kid, value means signature
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub proofs: Vec<IdProof>,
    // Key means id, value means signature
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub external_proofs: Vec<IdProofReceipt>,
}

impl IdEventReceipt {
    pub fn verify_proofs(&self, signers: &BTreeSet<IdSigner>) -> Result<(), IdEventError> {
        for proof in &self.proofs {
            let kid = Cid::from_str(&proof.key_id)?;
            let signer = signers
                .iter()
                .find(|s| s.id == proof.key_id)
                .ok_or(IdEventError::LackOfMinProofs)?;
            kid.ensure(&signer.public_key, vec![ED_CODE])?;

            match kid.codec() {
                ED_CODE => ed25519::verify(&signer.public_key, &self.payload, &proof.signature)?,
                _ => {
                    return Err(IdEventError::InvalidSigner(
                        "Unsupported key type".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }

    fn verify_delegation_proofs(
        receipt: &IdEventReceipt,
        delegators: &Vec<String>,
    ) -> Result<(), IdEventError> {
        for delegator in delegators {
            let proof = receipt
                .external_proofs
                .iter()
                .find(|p| p.id == *delegator)
                .ok_or(IdEventError::LackOfMinProofs)?;
            let created_at: DateTime<Utc> = proof
                .created_at
                .parse()
                .map_err(|_| IdEventError::invalid_proof(&proof.id, "Invalid created_at"))?;
            let cid = Cid::try_from(proof.content_id.clone())?;
            cid.ensure(&receipt.payload, vec![CBOR_CODE])?;
            let data = cbor!({
                "id" => proof.id.clone(),
                "purpose" => "delegation",
                "version" => proof.version.clone(),
                "key_id" => proof.key_id.clone(),
                "created_at" => created_at.timestamp(),
                "content_id" => proof.content_id,
            })
            .map_err(|_| CommonError::EncodeError)?;
            let data_bytes = idp2p_common::cbor::encode(&data);
            crate::host::verify_proof(&proof, &data_bytes).map_err(|_| {
                IdEventError::invalid_proof(&proof.id, "Delegated proof verification failed")
            })?;
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
        let valid_from: DateTime<Utc> = VALID_FROM.parse().expect("Invalid date format");
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

/*
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::IdProof;
    use ed25519_dalek::{SigningKey, VerifyingKey, ed25519::signature::SignerMut};
    use idp2p_common::{CBOR_CODE, cbor as cbor_util};
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

    fn base_state_with_signer(id: &str, pubkey: &[u8]) -> IdState {
        IdState {
            id: "did:idp2p:test".into(),
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
            claims: vec![],
            revoked: false,
            revoked_at: None,
        }
    }

    fn sign_receipt(payload: &[u8], signer_id: &str, sk: &mut SigningKey) -> IdProof {
        IdProof {
            key_id: signer_id.to_string(),
            created_at: Utc::now().to_rfc3339(),
            signature: sk.sign(payload).to_vec(),
        }
    }

    #[test]
    fn test_interaction_add_and_revoke_claim() {
        let (sid, vk, mut sk) = create_signer();
        let mut state = base_state_with_signer(&sid, vk.as_bytes());

        // Pre-populate a claim to be revoked
        state.claims.push(crate::types::IdClaim {
            key: "role".into(),
            values: vec![crate::types::IdClaimValue {
                id: "admin".into(),
                valid_from: Utc::now().to_rfc3339(),
                valid_until: None,
                payload: None,
            }],
        });

        // Prepare event that adds a new claim and revokes existing one
        let event = IdEvent {
            version: VERSION.into(),
            timestamp: Utc::now().timestamp(),
            component: Cid::default(),
            previous: state.event_id.clone(),
            body: Interaction {
                new_claims: {
                    let mut s = BTreeSet::new();
                    s.insert(IdClaimCreateEvent {
                        key: "email".into(),
                        id: "primary".into(),
                        payload: Some(b"user@example.com".to_vec()),
                    });
                    s
                },
                revoked_claims: {
                    let mut s = BTreeSet::new();
                    s.insert(IdClaimRevokeEvent {
                        key: "role".into(),
                        id: "admin".into(),
                    });
                    s
                },
            },
        };

        let payload = cbor_util::encode(&event);
        let receipt = IdEventReceipt {
            id: Cid::create(CBOR_CODE, &payload).unwrap().to_string(),
            version: VERSION.into(),
            created_at: Utc::now().to_rfc3339(),
            payload: payload.clone(),
            proofs: vec![sign_receipt(&payload, &sid, &mut sk)],
            external_proofs: vec![],
        };

        let updated = verify(&receipt, &mut state).expect("event verification should pass");

        // Check event id updated
        assert_eq!(updated.event_id, receipt.id);

        // New claim added
        let email_claim = updated
            .claims
            .iter()
            .find(|c| c.key == "email")
            .expect("email claim exists");
        assert_eq!(email_claim.values.len(), 1);
        assert_eq!(email_claim.values[0].id, "primary");
        assert!(email_claim.values[0].valid_until.is_none());

        // Existing claim revoked
        let role_claim = updated
            .claims
            .iter()
            .find(|c| c.key == "role")
            .expect("role claim exists");
        assert_eq!(role_claim.values.len(), 1);
        assert_eq!(role_claim.values[0].id, "admin");
        assert!(role_claim.values[0].valid_until.is_some());
    }

    #[test]
    fn test_revocation_event_marks_revoked() {
        let (sid, vk, mut sk) = create_signer();
        let mut state = base_state_with_signer(&sid, vk.as_bytes());

        // Prepare Revocation event
        let mut revealed = BTreeSet::new();
        revealed.insert(super::IdSigner {
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
        let payload = cbor_util::encode(&event);
        let receipt = IdEventReceipt {
            id: Cid::create(CBOR_CODE, &payload).unwrap().to_string(),
            version: VERSION.into(),
            created_at: Utc::now().to_rfc3339(),
            payload: payload.clone(),
            proofs: vec![sign_receipt(&payload, &sid, &mut sk)],
            external_proofs: vec![],
        };

        let updated = verify(&receipt, &mut state).expect("revocation should pass");
        assert!(updated.revoked);
        assert!(updated.revoked_at.is_some());
        assert_eq!(updated.event_id, receipt.id);
    }

    #[test]
    fn test_migration_event_sets_next_id() {
        let (sid, vk, mut sk) = create_signer();
        let mut state = base_state_with_signer(&sid, vk.as_bytes());

        let mut revealed = BTreeSet::new();
        revealed.insert(super::IdSigner {
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
        let payload = cbor_util::encode(&event);
        let receipt = IdEventReceipt {
            id: Cid::create(CBOR_CODE, &payload).unwrap().to_string(),
            version: VERSION.into(),
            created_at: Utc::now().to_rfc3339(),
            payload: payload.clone(),
            proofs: vec![sign_receipt(&payload, &sid, &mut sk)],
            external_proofs: vec![],
        };

        let updated = verify(&receipt, &mut state).expect("migration should pass");
        assert_eq!(updated.next_id, Some("did:idp2p:new".into()));
        assert_eq!(updated.event_id, receipt.id);
    }

    #[test]
    fn test_rotation_updates_thresholds_and_next_signers() {
        let (sid, vk, mut sk) = create_signer();
        let mut state = base_state_with_signer(&sid, vk.as_bytes());

        // Must reveal a signer from state.next_signers
        let mut revealed = BTreeSet::new();
        revealed.insert(super::IdSigner {
            id: sid.clone(),
            public_key: vk.as_bytes().to_vec(),
        });
        // No new signers; keep same next_signers
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
        let payload = cbor_util::encode(&event);
        let receipt = IdEventReceipt {
            id: Cid::create(CBOR_CODE, &payload).unwrap().to_string(),
            version: VERSION.into(),
            created_at: Utc::now().to_rfc3339(),
            payload: payload.clone(),
            proofs: vec![sign_receipt(&payload, &sid, &mut sk)],
            external_proofs: vec![],
        };

        let updated = verify(&receipt, &mut state).expect("rotation should pass");
        assert_eq!(updated.threshold, 1);
        assert_eq!(updated.next_threshold, 1);
        assert_eq!(updated.next_signers, vec![sid]);
        assert_eq!(updated.event_id, receipt.id);
    }

    #[test]
    fn test_event_updates_event_timestamp() {
        let (sid, vk, mut sk) = create_signer();
        let mut state = base_state_with_signer(&sid, vk.as_bytes());
        // Use a fixed timestamp to validate formatting
        let ts = Utc::now().timestamp();
        let expected_ts: String = String::try_from(crate::verifier::utils::Timestamp(ts)).unwrap();

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
        let payload = cbor_util::encode(&event);
        let receipt = IdEventReceipt {
            id: Cid::create(CBOR_CODE, &payload).unwrap().to_string(),
            version: VERSION.into(),
            created_at: Utc::now().to_rfc3339(),
            payload: payload.clone(),
            proofs: vec![sign_receipt(&payload, &sid, &mut sk)],
            external_proofs: vec![],
        };

        let updated = verify(&receipt, &mut state).expect("event verification should pass");
        assert_eq!(updated.event_timestamp, expected_ts);
    }

    #[test]
    fn test_invalid_version() {
        let (sid, vk, mut sk) = create_signer();
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
        let payload = cbor_util::encode(&event);
        let receipt = IdEventReceipt {
            id: Cid::create(CBOR_CODE, &payload).unwrap().to_string(),
            version: VERSION.into(),
            created_at: Utc::now().to_rfc3339(),
            payload: payload.clone(),
            proofs: vec![sign_receipt(&payload, &sid, &mut sk)],
            external_proofs: vec![],
        };
        let err = verify(&receipt, &mut state).unwrap_err();
        assert!(matches!(err, IdEventError::UnsupportedVersion));
    }

    #[test]
    fn test_invalid_timestamp() {
        let (sid, vk, mut sk) = create_signer();
        let mut state = base_state_with_signer(&sid, vk.as_bytes());
        let event = IdEvent {
            version: VERSION.into(),
            timestamp: 0, // before VALID_FROM
            component: Cid::default(),
            previous: state.event_id.clone(),
            body: Interaction {
                new_claims: BTreeSet::new(),
                revoked_claims: BTreeSet::new(),
            },
        };
        let payload = cbor_util::encode(&event);
        let receipt = IdEventReceipt {
            id: Cid::create(CBOR_CODE, &payload).unwrap().to_string(),
            version: VERSION.into(),
            created_at: Utc::now().to_rfc3339(),
            payload: payload.clone(),
            proofs: vec![sign_receipt(&payload, &sid, &mut sk)],
            external_proofs: vec![],
        };
        let err = verify(&receipt, &mut state).unwrap_err();
        assert!(matches!(err, IdEventError::InvalidTimestamp));
    }

    #[test]
    fn test_interaction_insufficient_proofs_for_threshold() {
        let (sid1, vk1, mut sk1) = create_signer();
        let (sid2, vk2, mut _sk2) = create_signer();
        // Prepare state requiring 2 proofs
        let mut state = base_state_with_signer(&sid1, vk1.as_bytes());
        // Add second signer to state but keep threshold 2
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
        let payload = cbor_util::encode(&event);
        let receipt = IdEventReceipt {
            id: Cid::create(CBOR_CODE, &payload).unwrap().to_string(),
            version: VERSION.into(),
            created_at: Utc::now().to_rfc3339(),
            payload: payload.clone(),
            proofs: vec![sign_receipt(&payload, &sid1, &mut sk1)], // only 1 proof
            external_proofs: vec![],
        };
        let err = verify(&receipt, &mut state).unwrap_err();
        assert!(matches!(err, IdEventError::LackOfMinProofs));
    }

    #[test]
    fn test_previous_mismatch() {
        let (sid, vk, mut sk) = create_signer();
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
        let payload = cbor_util::encode(&event);
        let receipt = IdEventReceipt {
            id: Cid::create(CBOR_CODE, &payload).unwrap().to_string(),
            version: VERSION.into(),
            created_at: Utc::now().to_rfc3339(),
            payload: payload.clone(),
            proofs: vec![sign_receipt(&payload, &sid, &mut sk)],
            external_proofs: vec![],
        };
        let err = verify(&receipt, &mut state).unwrap_err();
        assert!(matches!(err, IdEventError::PreviousNotMatch));
    }

    #[test]
    fn test_cid_payload_mismatch() {
        let (sid, vk, mut sk) = create_signer();
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
        let payload = cbor_util::encode(&event);
        // Intentionally compute id using different payload
        let wrong_id = Cid::create(CBOR_CODE, b"different").unwrap().to_string();
        let receipt = IdEventReceipt {
            id: wrong_id,
            version: VERSION.into(),
            created_at: Utc::now().to_rfc3339(),
            payload: payload.clone(),
            proofs: vec![sign_receipt(&payload, &sid, &mut sk)],
            external_proofs: vec![],
        };
        let err = verify(&receipt, &mut state).unwrap_err();
        // Error originates from cid.ensure => CommonError wrapped
        assert!(matches!(err, IdEventError::CommonError(_)));
    }

    #[test]
    fn test_rotation_invalid_next_signer_codec() {
        let (sid, vk, mut sk) = create_signer();
        let mut state = base_state_with_signer(&sid, vk.as_bytes());
        let mut revealed = BTreeSet::new();
        revealed.insert(super::IdSigner {
            id: sid.clone(),
            public_key: vk.as_bytes().to_vec(),
        });
        // Create a non-ED25519 CID (use CBOR_CODE)
        let bad_next = Cid::create(CBOR_CODE, b"x").unwrap().to_string();
        let mut next_signers = BTreeSet::new();
        next_signers.insert(bad_next.clone());
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
        let payload = cbor_util::encode(&event);
        let receipt = IdEventReceipt {
            id: Cid::create(CBOR_CODE, &payload).unwrap().to_string(),
            version: VERSION.into(),
            created_at: Utc::now().to_rfc3339(),
            payload: payload.clone(),
            proofs: vec![sign_receipt(&payload, &sid, &mut sk)],
            external_proofs: vec![],
        };
        let err = verify(&receipt, &mut state).unwrap_err();
        assert!(matches!(err, IdEventError::InvalidNextSigner(s) if s == bad_next));
    }

    #[test]
    fn test_rotation_revealed_not_in_next_signers() {
        let (sid1, vk1, mut sk1) = create_signer();
        let (sid2, vk2, _) = create_signer();
        let mut state = base_state_with_signer(&sid1, vk1.as_bytes());
        // state.next_signers contains sid1 only. Reveal sid2 to trigger error
        let mut revealed = BTreeSet::new();
        revealed.insert(super::IdSigner {
            id: sid2.clone(),
            public_key: vk2.as_bytes().to_vec(),
        });
        let mut next_signers = BTreeSet::new();
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
        let payload = cbor_util::encode(&event);
        let receipt = IdEventReceipt {
            id: Cid::create(CBOR_CODE, &payload).unwrap().to_string(),
            version: VERSION.into(),
            created_at: Utc::now().to_rfc3339(),
            payload: payload.clone(),
            proofs: vec![sign_receipt(&payload, &sid1, &mut sk1)],
            external_proofs: vec![],
        };
        let err = verify(&receipt, &mut state).unwrap_err();
        assert!(matches!(err, IdEventError::ThresholdNotMatch));
    }

    #[test]
    fn test_rotation_insufficient_next_signers() {
        let (sid, vk, mut sk) = create_signer();
        let mut state = base_state_with_signer(&sid, vk.as_bytes());
        let mut revealed = BTreeSet::new();
        revealed.insert(super::IdSigner {
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
        let payload = cbor_util::encode(&event);
        let receipt = IdEventReceipt {
            id: Cid::create(CBOR_CODE, &payload).unwrap().to_string(),
            version: VERSION.into(),
            created_at: Utc::now().to_rfc3339(),
            payload: payload.clone(),
            proofs: vec![sign_receipt(&payload, &sid, &mut sk)],
            external_proofs: vec![],
        };
        let err = verify(&receipt, &mut state).unwrap_err();
        assert!(matches!(err, IdEventError::NextThresholdNotMatch));
    }

    #[test]
    fn test_revocation_threshold_not_met() {
        let (sid1, vk1, mut sk1) = create_signer();
        let (sid2, vk2, _) = create_signer();
        // Prepare state requiring 2 revealed signers
        let mut state = base_state_with_signer(&sid1, vk1.as_bytes());
        state.next_threshold = 2;
        state.next_signers = vec![sid1.clone(), sid2.clone()];
        let mut revealed = BTreeSet::new();
        revealed.insert(super::IdSigner {
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
        let payload = cbor_util::encode(&event);
        let receipt = IdEventReceipt {
            id: Cid::create(CBOR_CODE, &payload).unwrap().to_string(),
            version: VERSION.into(),
            created_at: Utc::now().to_rfc3339(),
            payload: payload.clone(),
            proofs: vec![sign_receipt(&payload, &sid1, &mut sk1)],
            external_proofs: vec![],
        };
        let err = verify(&receipt, &mut state).unwrap_err();
        assert!(matches!(err, IdEventError::ThresholdNotMatch));
    }

    #[test]
    fn test_migration_signer_not_in_next_signers() {
        let (sid1, vk1, mut sk1) = create_signer();
        let (sid2, vk2, _) = create_signer();
        let mut state = base_state_with_signer(&sid1, vk1.as_bytes());
        // Reveal sid2 which is not in next_signers
        let mut revealed = BTreeSet::new();
        revealed.insert(super::IdSigner {
            id: sid2.clone(),
            public_key: vk2.as_bytes().to_vec(),
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
        let payload = cbor_util::encode(&event);
        let receipt = IdEventReceipt {
            id: Cid::create(CBOR_CODE, &payload).unwrap().to_string(),
            version: VERSION.into(),
            created_at: Utc::now().to_rfc3339(),
            payload: payload.clone(),
            proofs: vec![sign_receipt(&payload, &sid1, &mut sk1)],
            external_proofs: vec![],
        };
        let err = verify(&receipt, &mut state).unwrap_err();
        assert!(matches!(err, IdEventError::ThresholdNotMatch));
    }
}
*/
