use alloc::collections::BTreeSet;
use alloc::str::FromStr;
use chrono::{DateTime, Utc};
use cid::Cid;
use idp2p_common::{CBOR_CODE, ED_CODE, bytes::Bytes, cid::CidExt, error::CommonError};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::{
    VALID_FROM, VERSION,
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
        let mut seen: BTreeSet<String> = BTreeSet::new();
        for proof in self.proofs.iter() {
            if !seen.insert(proof.key_id.clone()) {
                return Err(IdEventError::invalid_proof(
                    &proof.key_id,
                    "duplicate proof",
                ));
            }
            match proof.purpose.as_str() {
                "id-delegation" => {
                    proof.verify(&self.payload, &signers)?;
                }
                _ => {
                    crate::host::verify_proof(&proof, &self.payload)
                        .map_err(|e| IdEventError::invalid_proof(&proof.key_id, &e.code))?;
                }
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
       
        let id_state = IdState {
            id: self.id.clone(),
            event_id: self.id.clone(),
            event_timestamp: timestamp.clone(),
            prior_id: inception.prior_id.clone(),
            next_id_proof: None,
            threshold: inception.threshold,
            next_threshold: inception.next_threshold,
            signers: inception
                .signers
                .clone()
                .into_iter()
                .map(|s| s.to_state(0, &timestamp))
                .collect(),
            current_signers: inception
                .signers
                .into_iter()
                .map(|signer| signer.id)
                .collect(),
            next_signers: inception.next_signers.into_iter().collect(),
            delegated_signers: vec![],
            merkle_proof: inception.merkle_proof,
            revoked: false,
            revoked_at: None,
        };
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
            Interaction { merkle_proof } => {
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
                state.merkle_proof = merkle_proof;
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
                    .map(|s| s.to_state(event.sn, &timestamp))
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
                next_id_proof,
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
                state.next_id_proof = Some(next_id_proof);
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
    use crate::internal::utils::Timestamp;
    use alloc::collections::BTreeSet;
    use chrono::{DateTime, Utc};
    use ed25519_dalek::{Signer as _, SigningKey, VerifyingKey};
    use ciborium::cbor;
    use idp2p_common::{cbor as common_cbor, cid::CidExt, CBOR_CODE, ED_CODE};
    use rand::rngs::OsRng;

    fn valid_timestamp() -> i64 {
        let valid_from: DateTime<Utc> = crate::VALID_FROM.parse().expect("valid timestamp");
        valid_from.timestamp() + 1
    }

    fn timestamp_string(ts: i64) -> String {
        String::try_from(Timestamp(ts)).expect("timestamp conversion")
    }

    fn create_signer() -> (String, VerifyingKey, SigningKey) {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        let id = Cid::create(ED_CODE, verifying_key.as_bytes())
            .expect("cid")
            .to_string();
        (id, verifying_key, signing_key)
    }

    fn sign_receipt(payload: &[u8], creator: &str, kid: &str, sk: &SigningKey) -> IdProof {
        let created = Utc::now();
        let created_str = created.to_rfc3339();
        let data = cbor!({
            "did" => creator.to_string(),
            "key_id" => kid.to_string(),
            "created" => created.timestamp(),
            "purpose" => "id-delegation",
            "payload" => payload.to_vec(),
        })
        .expect("cbor data");

        let data_bytes = common_cbor::encode(&data);
        let signature = sk.sign(&data_bytes);

        IdProof {
            id: Cid::create(CBOR_CODE, payload)
                .expect("proof cid")
                .to_string(),
            did: creator.into(),
            key_id: kid.into(),
            created: created_str,
            purpose: "id-delegation".into(),
            signature: signature.to_vec(),
            previous: None,
        }
    }

    fn base_state_with_signer(id: &str, pubkey: &[u8]) -> IdState {
        let ts = valid_timestamp();
        let ts_str = timestamp_string(ts);
        IdState {
            id: Cid::create(CBOR_CODE, b"idp2p-test")
                .expect("state cid")
                .to_string(),
            event_id: Cid::create(CBOR_CODE, b"previous-event")
                .expect("event cid")
                .to_string(),
            event_timestamp: ts_str.clone(),
            prior_id: None,
            next_id_proof: None,
            threshold: 1,
            next_threshold: 1,
            signers: vec![crate::types::IdSigner {
                id: id.to_string(),
                public_key: pubkey.to_vec(),
                valid_from_sn: 0,
                valid_until_sn: None,
                valid_from: ts_str.clone(),
                valid_until: None,
            }],
            current_signers: vec![id.to_string()],
            next_signers: vec![id.to_string()],
            delegated_signers: vec![],
            merkle_proof: "existing-merkle-proof".into(),
            revoked: false,
            revoked_at: None,
        }
    }

    #[test]
    fn test_interaction_event_success() {
        let (sid, vk, sk) = create_signer();
        let mut state = base_state_with_signer(&sid, vk.as_bytes());
        let ts = valid_timestamp();

        let event = IdEvent {
            sn: 1,
            version: VERSION.into(),
            patch: Cid::default(),
            timestamp: ts,
            previous: state.event_id.clone(),
            body: Interaction {
                merkle_proof: "new-proof".into(),
            },
        };
        let payload = common_cbor::encode(&event);
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
        assert_eq!(updated.event_timestamp, timestamp_string(ts));
    }

    #[test]
    fn test_event_updates_event_timestamp() {
        let (sid, vk, sk) = create_signer();
        let mut state = base_state_with_signer(&sid, vk.as_bytes());
        let ts = valid_timestamp();

        let event = IdEvent {
            sn: 2,
            version: VERSION.into(),
            patch: Cid::default(),
            timestamp: ts,
            previous: state.event_id.clone(),
            body: Interaction {
                merkle_proof: "proof".into(),
            },
        };
        let payload = common_cbor::encode(&event);
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
        assert_eq!(updated.event_timestamp, timestamp_string(ts));
    }

    #[test]
    fn test_invalid_version() {
        let (sid, vk, sk) = create_signer();
        let mut state = base_state_with_signer(&sid, vk.as_bytes());

        let event = IdEvent {
            sn: 3,
            version: "0.9".into(),
            patch: Cid::default(),
            timestamp: valid_timestamp(),
            previous: state.event_id.clone(),
            body: Interaction {
                merkle_proof: "proof".into(),
            },
        };
        let payload = common_cbor::encode(&event);
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
        let ts = valid_timestamp() - 10;

        let event = IdEvent {
            sn: 4,
            version: VERSION.into(),
            patch: Cid::default(),
            timestamp: ts,
            previous: state.event_id.clone(),
            body: Interaction {
                merkle_proof: "proof".into(),
            },
        };
        let payload = common_cbor::encode(&event);
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
        let ts = valid_timestamp();
        state.signers.push(crate::types::IdSigner {
            id: sid2.clone(),
            public_key: vk2.as_bytes().to_vec(),
            valid_from_sn: 0,
            valid_until_sn: None,
            valid_from: timestamp_string(ts),
            valid_until: None,
        });
        state.current_signers.push(sid2.clone());
        state.threshold = 2;

        let event = IdEvent {
            sn: 5,
            version: VERSION.into(),
            patch: Cid::default(),
            timestamp: ts,
            previous: state.event_id.clone(),
            body: Interaction {
                merkle_proof: "proof".into(),
            },
        };
        let payload = common_cbor::encode(&event);
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
            sn: 6,
            version: VERSION.into(),
            patch: Cid::default(),
            timestamp: valid_timestamp(),
            previous: "wrong-previous".into(),
            body: Interaction {
                merkle_proof: "proof".into(),
            },
        };
        let payload = common_cbor::encode(&event);
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
    fn test_rotation_event_success() {
        let (sid1, vk1, sk1) = create_signer();
        let (sid2, vk2, sk2) = create_signer();
        let mut state = base_state_with_signer(&sid1, vk1.as_bytes());
        state.next_signers = vec![sid1.clone()];
        state.next_threshold = 1;
        let ts = valid_timestamp();

        let mut revealed = BTreeSet::new();
        revealed.insert(InternalSigner {
            id: sid1.clone(),
            public_key: vk1.as_bytes().to_vec(),
        });
        let mut new_signers = BTreeSet::new();
        new_signers.insert(InternalSigner {
            id: sid2.clone(),
            public_key: vk2.as_bytes().to_vec(),
        });
        let mut next_signers = BTreeSet::new();
        next_signers.insert(sid2.clone());

        let event = IdEvent {
            sn: 7,
            version: VERSION.into(),
            patch: Cid::default(),
            timestamp: ts,
            previous: state.event_id.clone(),
            body: Rotation {
                threshold: 1,
                next_threshold: 1,
                revealed_signers: revealed,
                new_signers: new_signers,
                next_signers: next_signers,
            },
        };
        let payload = common_cbor::encode(&event);
        let receipt = IdEventReceipt {
            id: Cid::create(CBOR_CODE, &payload).unwrap().to_string(),
            version: VERSION.into(),
            created_at: Utc::now().to_rfc3339(),
            payload: payload.clone(),
            proofs: vec![
                sign_receipt(&payload, &state.id, &sid1, &sk1),
                sign_receipt(&payload, &state.id, &sid2, &sk2),
            ],
        };

        let updated = receipt
            .verify_event(&mut state)
            .expect("rotation should pass");
        let expected_ts = timestamp_string(ts);
        let original = updated
            .signers
            .iter()
            .find(|s| s.id == sid1)
            .expect("original signer");
        assert_eq!(updated.threshold, 1);
        assert_eq!(updated.next_signers, vec![sid2.clone()]);
        assert_eq!(original.valid_until.as_ref(), Some(&expected_ts));
        assert!(updated
            .signers
            .iter()
            .any(|s| s.id == sid2 && s.valid_from_sn == event.sn));
    }

    #[test]
    fn test_rotation_invalid_next_signer_codec() {
        let (sid, vk, sk) = create_signer();
        let mut state = base_state_with_signer(&sid, vk.as_bytes());
        state.next_signers = vec![sid.clone()];
        state.next_threshold = 1;

        let mut revealed = BTreeSet::new();
        revealed.insert(InternalSigner {
            id: sid.clone(),
            public_key: vk.as_bytes().to_vec(),
        });
        let invalid_next = Cid::create(CBOR_CODE, vk.as_bytes())
            .unwrap()
            .to_string();
        let mut next_signers = BTreeSet::new();
        next_signers.insert(invalid_next);

        let event = IdEvent {
            sn: 8,
            version: VERSION.into(),
            patch: Cid::default(),
            timestamp: valid_timestamp(),
            previous: state.event_id.clone(),
            body: Rotation {
                threshold: 1,
                next_threshold: 1,
                revealed_signers: revealed,
                new_signers: BTreeSet::new(),
                next_signers,
            },
        };
        let payload = common_cbor::encode(&event);
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
    fn test_revocation_event_success() {
        let (sid, vk, sk) = create_signer();
        let mut state = base_state_with_signer(&sid, vk.as_bytes());
        state.next_signers = vec![sid.clone()];
        state.next_threshold = 1;
        let ts = valid_timestamp();

        let mut revealed = BTreeSet::new();
        revealed.insert(InternalSigner {
            id: sid.clone(),
            public_key: vk.as_bytes().to_vec(),
        });
        let event = IdEvent {
            sn: 9,
            version: VERSION.into(),
            patch: Cid::default(),
            timestamp: ts,
            previous: state.event_id.clone(),
            body: Revocation {
                revealed_signers: revealed,
            },
        };
        let payload = common_cbor::encode(&event);
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
        assert!(updated.next_signers.is_empty());
        assert_eq!(updated.event_id, receipt.id);
    }

    #[test]
    fn test_revocation_threshold_not_met() {
        let (sid1, vk1, sk1) = create_signer();
        let (sid2, _vk2, _sk2) = create_signer();
        let mut state = base_state_with_signer(&sid1, vk1.as_bytes());
        state.next_signers = vec![sid1.clone(), sid2.clone()];
        state.next_threshold = 2;

        let mut revealed = BTreeSet::new();
        revealed.insert(InternalSigner {
            id: sid1.clone(),
            public_key: vk1.as_bytes().to_vec(),
        });
        let event = IdEvent {
            sn: 10,
            version: VERSION.into(),
            patch: Cid::default(),
            timestamp: valid_timestamp(),
            previous: state.event_id.clone(),
            body: Revocation {
                revealed_signers: revealed,
            },
        };
        let payload = common_cbor::encode(&event);
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
    fn test_migration_event_sets_next_id_proof() {
        let (sid, vk, sk) = create_signer();
        let mut state = base_state_with_signer(&sid, vk.as_bytes());
        state.next_signers = vec![sid.clone()];
        state.next_threshold = 1;
        let ts = valid_timestamp();

        let mut revealed = BTreeSet::new();
        revealed.insert(InternalSigner {
            id: sid.clone(),
            public_key: vk.as_bytes().to_vec(),
        });
        let next_id_proof = "did:idp2p:new-proof";
        let event = IdEvent {
            sn: 11,
            version: VERSION.into(),
            patch: Cid::default(),
            timestamp: ts,
            previous: state.event_id.clone(),
            body: Migration {
                revealed_signers: revealed,
                next_id_proof: next_id_proof.into(),
            },
        };
        let payload = common_cbor::encode(&event);
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
        assert_eq!(updated.next_id_proof, Some(next_id_proof.into()));
        assert!(updated.next_signers.is_empty());
        assert_eq!(updated.event_id, receipt.id);
    }

    #[test]
    fn test_duplicate_proofs_rejected() {
        let (sid, vk, sk) = create_signer();
        let mut state = base_state_with_signer(&sid, vk.as_bytes());
        let event = IdEvent {
            sn: 12,
            version: VERSION.into(),
            patch: Cid::default(),
            timestamp: valid_timestamp(),
            previous: state.event_id.clone(),
            body: Interaction {
                merkle_proof: "proof".into(),
            },
        };
        let payload = common_cbor::encode(&event);
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
            sn: 13,
            version: VERSION.into(),
            patch: Cid::default(),
            timestamp: valid_timestamp(),
            previous: state.event_id.clone(),
            body: Interaction {
                merkle_proof: "proof".into(),
            },
        };
        let payload = common_cbor::encode(&event);
        let mut proof = sign_receipt(&payload, &state.id, &sid, &sk);
        proof.created = "not-a-date".into();
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
            sn: 14,
            version: VERSION.into(),
            patch: Cid::default(),
            timestamp: valid_timestamp(),
            previous: state.event_id.clone(),
            body: Interaction {
                merkle_proof: "proof".into(),
            },
        };
        let payload = common_cbor::encode(&event);
        let mut proof = sign_receipt(&payload, &state.id, &sid, &sk);
        proof.signature[0] ^= 0xFF;
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
            sn: 15,
            version: VERSION.into(),
            patch: Cid::default(),
            timestamp: valid_timestamp(),
            previous: state.event_id.clone(),
            body: Interaction {
                merkle_proof: "proof".into(),
            },
        };
        let payload = common_cbor::encode(&event);
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
