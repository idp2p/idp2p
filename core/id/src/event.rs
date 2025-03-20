use alloc::str::FromStr;

use crate::{
    error::IdEventError,
    types::{IdClaimEvent, IdSigner},
};
use idp2p_common::{cbor, identifier::Identifier};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IdRotation {
    pub signers: Vec<IdSigner>,
    pub next_threshold: u8,
    pub next_signers: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum IdEventKind {
    /// Should be signed with current keys
    Interaction(Vec<IdClaimEvent>),

    /// Should be signed with next keys
    Rotation(IdRotation),

    /// Should be signed with next keys
    Migration(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IdEvent {
    /// Timestamp of event
    pub timestamp: i64,

    /// Previous event id
    pub previous: String,

    /// Event payload
    pub payload: IdEventKind,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersistedIdEvent {
    id: String,
    payload: Vec<u8>,
    proofs: Vec<PersistedIdProof>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersistedIdProof {
    kid: String,
    sig: Vec<u8>,
    pk: Option<Vec<u8>>,
}

impl TryFrom<&PersistedIdEvent> for IdEvent {
    type Error = IdEventError;

    fn try_from(value: &PersistedIdEvent) -> Result<Self, Self::Error> {
        let id = Identifier::from_str(value.id.as_str())
            .map_err(|e| IdEventError::InvalidEventId(e.to_string()))?;
        id.ensure(&value.payload)
            .map_err(|e| IdEventError::InvalidEventId(e.to_string()))?;

        if id.kind != "event" {
            return Err(IdEventError::InvalidEventId(id.to_string()));
        }

        let event: IdEvent =
            cbor::decode(&value.payload).map_err(|_| IdEventError::InvalidPayload)?;
        Ok(event)
    }
}

pub(crate) fn verify(state: &[u8], payload: &[u8]) -> Result<Vec<u8>, IdEventError> {
    todo!()
}
/*
impl PersistedIdEvent {
    pub(crate) fn verify(
        &self,
        projection: &mut IdProjection,
    ) -> Result<IdProjection, IdEventError> {
        let event: IdEvent = self.try_into()?;

        // Timestamp check
        if event.timestamp < TIMESTAMP {
            return Err(IdEventError::InvalidTimestamp);
        }

        // Previous event check
        if event.previous != projection.event_id {
            return Err(IdEventError::PreviousNotMatch);
        }

        // Proof verification
        let mut signers = HashSet::new();
        for proof in &self.proofs {
            let sid = Id::from_str(proof.id.as_str()).map_err(|e| {
                IdEventError::InvalidProof(IdError {
                    id: proof.id.clone(),
                    reason: e.to_string(),
                })
            })?;

            sid.ensure(&proof.pk).map_err(|e| {
                IdEventError::InvalidProof(IdError {
                    id: proof.id.clone(),
                    reason: e.to_string(),
                })
            })?;

            if !signers.insert(proof.id.clone()) {
                return Err(IdEventError::InvalidProof(IdError {
                    id: proof.id.clone(),
                    reason: "duplicate-proof".to_string(),
                }));
            }

            verify(&proof.pk, &self.payload, &proof.sig).map_err(|e| {
                IdEventError::InvalidProof(IdError {
                    id: proof.id.clone(),
                    reason: e.to_string(),
                })
            })?;
        }

        match event.payload {
            Interaction(claims) => {
                if (signers.len() as u8) < projection.threshold {
                    return Err(IdEventError::LackOfMinProofs);
                }

                // Validate that all signers are recognized
                for signer in &signers {
                    if !projection.signers.iter().any(|s| s.id == *signer) {
                        return Err(IdEventError::InvalidProof(IdError {
                            id: signer.clone(),
                            reason: "signer-not-found".to_string(),
                        }));
                    }
                }

                // Process each claim
                for claim in claims {
                    projection.claims.push(claim);
                }
            }
            Rotation(id_rotation) => {
                for signer in &signers {
                    if !projection.next_signers.iter().any(|s| s == signer) {
                        return Err(IdEventError::InvalidProof(IdError {
                            id: signer.clone(),
                            reason: "signer-not-authorized".to_string(),
                        }));
                    }
                }

                // Signer check
                //
                let total_signers = id_rotation.signers.len() as u8;
                if total_signers < projection.threshold {
                    return Err(IdEventError::ThresholdNotMatch);
                }
                let mut signers = vec![];
                for signer in &id_rotation.signers {
                    let signer_id = Id::from_str(signer.id.as_str()).map_err(|e| {
                        IdEventError::InvalidSigner(IdError {
                            id: signer.id.clone(),
                            reason: e.to_string(),
                        })
                    })?;
                    if signer_id.kind != "signer" {
                        return Err(IdEventError::InvalidSigner(IdError {
                            id: signer.id.clone(),
                            reason: "invalid-signer-kind".to_string(),
                        }));
                    }
                    signer_id.ensure(&signer.public_key).map_err(|e| {
                        IdEventError::InvalidSigner(IdError {
                            id: signer.id.clone(),
                            reason: e.to_string(),
                        })
                    })?;
                    if signers.contains(signer) {
                        return Err(IdEventError::InvalidSigner(IdError {
                            id: signer.id.clone(),
                            reason: "duplicate-signer".to_string(),
                        }));
                    }
                    projection.all_signers.push(signer.id.clone());
                    signers.push(signer.to_owned());
                }

                // Next Signer check
                //
                let total_next_signers = id_rotation.next_signers.len() as u8;
                if total_next_signers < id_rotation.next_threshold {
                    return Err(IdEventError::ThresholdNotMatch);
                }
                let mut next_signers = vec![];
                for next_signer in &id_rotation.next_signers {
                    let next_signer_id = Id::from_str(next_signer.as_str()).map_err(|e| {
                        IdEventError::InvalidNextSigner(IdError {
                            id: next_signer.clone(),
                            reason: e.to_string(),
                        })
                    })?;
                    if next_signer_id.kind != "signer" {
                        return Err(IdEventError::InvalidNextSigner(IdError {
                            id: next_signer.clone(),
                            reason: "invalid-next-signer-kind".to_string(),
                        }));
                    }
                    if next_signers.contains(next_signer) {
                        return Err(IdEventError::InvalidNextSigner(IdError {
                            id: next_signer.clone(),
                            reason: "duplicate-next-signer".to_string(),
                        }));
                    }
                    projection.all_signers.push(next_signer.clone());
                    next_signers.push(next_signer.to_owned());
                }

                // Update the signers in the projection
                projection.signers = id_rotation.signers.clone();
            }
            Migration(next_id) => {
                for signer in &signers {
                    if !projection.next_signers.iter().any(|s| s == signer) {
                        return Err(IdEventError::InvalidProof(IdError {
                            id: signer.clone(),
                            reason: "signer-not-authorized".to_string(),
                        }));
                    }
                }
                // Validate the new delegated ID
                let delegated_id = Id::from_str(next_id.as_str())
                    .map_err(|e| IdEventError::Other("invalid-next-id".to_string()))?;

                delegated_id
                    .ensure(&self.payload)
                    .map_err(|e| IdEventError::Other("invalid-next-id".to_string()))?;

                // Update the projection with the new delegated ID
                projection.next_id = Some(next_id);
            }
        }

        // Update the view with the new event ID
        projection.event_id = self.id.clone();

        Ok(projection.to_owned())
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    /*
        #[test]
        fn test_verify_successful_interaction() {
            // Arrange
            let mut projection = IdProjection::new(
                "prev_event".to_string(),
                1,
                vec![Signer {
                    id: "signer1".to_string(),
                    public_key: vec![1, 2, 3],
                }],
            );

            let proofs = vec![Proof {
                id: "signer1".to_string(),
                pk: vec![1, 2, 3],
                sig: b"valid".to_vec(),
            }];

            let persisted_event = PersistedIdEvent {
                id: "event2".to_string(),
                payload: b"interaction".to_vec(),
                previous: "prev_event".to_string(),
                timestamp: TIMESTAMP + 100,
                proofs,
            };

            // Act
            let result = persisted_event.verify(&mut projection);

            // Assert
            assert!(result.is_ok());
            let updated_projection = result.unwrap();
            assert_eq!(updated_projection.event_id, "event2");
            // Assuming the interaction adds a claim, verify the claim exists
            // assert_eq!(updated_projection.claims.len(), 1);
            // assert_eq!(updated_projection.claims[0].id, "claim1");
        }

        #[test]
        fn test_verify_invalid_timestamp() {
            // Arrange
            let mut projection = IdProjection::new(
                "prev_event".to_string(),
                1,
                vec![Signer {
                    id: "signer1".to_string(),
                    public_key: vec![1, 2, 3],
                }],
            );

            let proofs = vec![Proof {
                id: "signer1".to_string(),
                pk: vec![1, 2, 3],
                sig: b"valid".to_vec(),
            }];

            let persisted_event = PersistedIdEvent {
                id: "event2".to_string(),
                payload: b"interaction".to_vec(),
                previous: "prev_event".to_string(),
                timestamp: TIMESTAMP - 100, // Invalid timestamp
                proofs,
            };

            // Act
            let result = persisted_event.verify(&mut projection);

            // Assert
            assert_eq!(result, Err(IdEventError::InvalidTimestamp));
        }

        #[test]
        fn test_verify_previous_event_mismatch() {
            // Arrange
            let mut projection = IdProjection::new(
                "prev_event".to_string(),
                1,
                vec![Signer {
                    id: "signer1".to_string(),
                    public_key: vec![1, 2, 3],
                }],
            );

            let proofs = vec![Proof {
                id: "signer1".to_string(),
                pk: vec![1, 2, 3],
                sig: b"valid".to_vec(),
            }];

            let persisted_event = PersistedIdEvent {
                id: "event2".to_string(),
                payload: b"interaction".to_vec(),
                previous: "wrong_prev".to_string(), // Mismatch
                timestamp: TIMESTAMP + 100,
                proofs,
            };

            // Act
            let result = persisted_event.verify(&mut projection);

            // Assert
            assert_eq!(result, Err(IdEventError::PreviousNotMatch));
        }

        #[test]
        fn test_verify_invalid_proof_id() {
            // Arrange
            let mut projection = IdProjection::new(
                "prev_event".to_string(),
                1,
                vec![Signer {
                    id: "signer1".to_string(),
                    public_key: vec![1, 2, 3],
                }],
            );

            let proofs = vec![Proof {
                id: "".to_string(), // Invalid ID
                pk: vec![1, 2, 3],
                sig: b"valid".to_vec(),
            }];

            let persisted_event = PersistedIdEvent {
                id: "event2".to_string(),
                payload: b"interaction".to_vec(),
                previous: "prev_event".to_string(),
                timestamp: TIMESTAMP + 100,
                proofs,
            };

            // Act
            let result = persisted_event.verify(&mut projection);

            // Assert
            assert_eq!(
                result,
                Err(IdEventError::InvalidProof(IdError {
                    id: "".to_string(),
                    reason: "Empty ID".to_string(),
                }))
            );
        }

        #[test]
        fn test_verify_invalid_proof_pk() {
            // Arrange
            let mut projection = IdProjection::new(
                "prev_event".to_string(),
                1,
                vec![Signer {
                    id: "signer1".to_string(),
                    public_key: vec![1, 2, 3],
                }],
            );

            let proofs = vec![Proof {
                id: "signer1".to_string(),
                pk: vec![], // Invalid PK
                sig: b"valid".to_vec(),
            }];

            let persisted_event = PersistedIdEvent {
                id: "event2".to_string(),
                payload: b"interaction".to_vec(),
                previous: "prev_event".to_string(),
                timestamp: TIMESTAMP + 100,
                proofs,
            };

            // Act
            let result = persisted_event.verify(&mut projection);

            // Assert
            assert_eq!(
                result,
                Err(IdEventError::InvalidProof(IdError {
                    id: "signer1".to_string(),
                    reason: "Invalid kind".to_string(),
                }))
            );
        }

        #[test]
        fn test_verify_duplicate_proofs() {
            // Arrange
            let mut projection = IdProjection::new(
                "prev_event".to_string(),
                2,
                vec![
                    Signer {
                        id: "signer1".to_string(),
                        public_key: vec![1, 2, 3],
                    },
                    Signer {
                        id: "signer2".to_string(),
                        public_key: vec![4, 5, 6],
                    },
                ],
            );

            let proofs = vec![
                Proof {
                    id: "signer1".to_string(),
                    pk: vec![1, 2, 3],
                    sig: b"valid".to_vec(),
                },
                Proof {
                    id: "signer1".to_string(), // Duplicate
                    pk: vec![1, 2, 3],
                    sig: b"valid".to_vec(),
                },
            ];

            let persisted_event = PersistedIdEvent {
                id: "event2".to_string(),
                payload: b"interaction".to_vec(),
                previous: "prev_event".to_string(),
                timestamp: TIMESTAMP + 100,
                proofs,
            };

            // Act
            let result = persisted_event.verify(&mut projection);

            // Assert
            assert_eq!(
                result,
                Err(IdEventError::InvalidProof(IdError {
                    id: "signer1".to_string(),
                    reason: "duplicate-proof".to_string(),
                }))
            );
        }

        #[test]
        fn test_verify_invalid_signature() {
            // Arrange
            let mut projection = IdProjection::new(
                "prev_event".to_string(),
                1,
                vec![Signer {
                    id: "signer1".to_string(),
                    public_key: vec![1, 2, 3],
                }],
            );

            let proofs = vec![Proof {
                id: "signer1".to_string(),
                pk: vec![1, 2, 3],
                sig: b"invalid".to_vec(), // Invalid signature
            }];

            let persisted_event = PersistedIdEvent {
                id: "event2".to_string(),
                payload: b"interaction".to_vec(),
                previous: "prev_event".to_string(),
                timestamp: TIMESTAMP + 100,
                proofs,
            };

            // Act
            let result = persisted_event.verify(&mut projection);

            // Assert
            assert_eq!(
                result,
                Err(IdEventError::InvalidProof(IdError {
                    id: "signer1".to_string(),
                    reason: "Invalid signature".to_string(),
                }))
            );
        }

        #[test]
        fn test_verify_insufficient_proofs() {
            // Arrange
            let mut projection = IdProjection::new(
                "prev_event".to_string(),
                2,
                vec![
                    Signer {
                        id: "signer1".to_string(),
                        public_key: vec![1, 2, 3],
                    },
                    Signer {
                        id: "signer2".to_string(),
                        public_key: vec![4, 5, 6],
                    },
                ],
            );

            let proofs = vec![Proof {
                id: "signer1".to_string(),
                pk: vec![1, 2, 3],
                sig: b"valid".to_vec(),
            }];

            let persisted_event = PersistedIdEvent {
                id: "event2".to_string(),
                payload: b"interaction".to_vec(),
                previous: "prev_event".to_string(),
                timestamp: TIMESTAMP + 100,
                proofs,
            };

            // Act
            let result = persisted_event.verify(&mut projection);

            // Assert
            assert_eq!(result, Err(IdEventError::LackOfMinProofs));
        }

        #[test]
        fn test_verify_signer_not_found() {
            // Arrange
            let mut projection = IdProjection::new(
                "prev_event".to_string(),
                1,
                vec![Signer {
                    id: "signer1".to_string(),
                    public_key: vec![1, 2, 3],
                }],
            );

            let proofs = vec![Proof {
                id: "unknown_signer".to_string(), // Not in projection.signers
                pk: vec![7, 8, 9],
                sig: b"valid".to_vec(),
            }];

            let persisted_event = PersistedIdEvent {
                id: "event2".to_string(),
                payload: b"interaction".to_vec(),
                previous: "prev_event".to_string(),
                timestamp: TIMESTAMP + 100,
                proofs,
            };

            // Act
            let result = persisted_event.verify(&mut projection);

            // Assert
            assert_eq!(
                result,
                Err(IdEventError::InvalidProof(IdError {
                    id: "unknown_signer".to_string(),
                    reason: "signer-not-found".to_string(),
                }))
            );
        }

        #[test]
        fn test_verify_interaction_add_and_remove_claims() {
            // Arrange
            let mut projection = IdProjection::new(
                "prev_event".to_string(),
                1,
                vec![Signer {
                    id: "signer1".to_string(),
                    public_key: vec![1, 2, 3],
                }],
            );

            // Initially, no claims
            assert_eq!(projection.claims.len(), 0);

            // First, add a claim
            let add_proofs = vec![Proof {
                id: "signer1".to_string(),
                pk: vec![1, 2, 3],
                sig: b"valid".to_vec(),
            }];

            let add_persisted_event = PersistedIdEvent {
                id: "event_add_claim".to_string(),
                payload: b"interaction".to_vec(),
                previous: "prev_event".to_string(),
                timestamp: TIMESTAMP + 100,
                proofs: add_proofs,
            };

            // Act
            let result_add = add_persisted_event.verify(&mut projection);

            // Assert
            assert!(result_add.is_ok());
            let updated_projection = result_add.unwrap();
            assert_eq!(updated_projection.event_id, "event_add_claim");
            // Since claims are empty in the payload, no claims should be added
            assert_eq!(updated_projection.claims.len(), 0);

            // Now, remove the claim (though no claims exist, this is to test retention)
            let remove_proofs = vec![Proof {
                id: "signer1".to_string(),
                pk: vec![1, 2, 3],
                sig: b"valid".to_vec(),
            }];

            let remove_persisted_event = PersistedIdEvent {
                id: "event_remove_claim".to_string(),
                payload: b"interaction".to_vec(),
                previous: "event_add_claim".to_string(),
                timestamp: TIMESTAMP + 200,
                proofs: remove_proofs,
            };

            // Act
            let result_remove = remove_persisted_event.verify(&mut projection);

            // Assert
            assert!(result_remove.is_ok());
            let updated_projection = result_remove.unwrap();
            assert_eq!(updated_projection.event_id, "event_remove_claim");
            // No claims to remove, so claims should remain the same
            assert_eq!(updated_projection.claims.len(), 0);
        }

        #[test]
        fn test_verify_rotation_success() {
            // Arrange
            let mut projection = IdProjection::new(
                "prev_event".to_string(),
                1,
                vec![
                    Signer {
                        id: "signer1".to_string(),
                        public_key: vec![1, 2, 3],
                    },
                    Signer {
                        id: "signer2".to_string(),
                        public_key: vec![4, 5, 6],
                    },
                ],
            );
            projection.next_signers = vec!["signer3".to_string()];
            projection.threshold = 2;

            let proofs = vec![Proof {
                id: "signer3".to_string(),
                pk: vec![7, 8, 9],
                sig: b"valid".to_vec(),
            }];

            let rotation = IdRotation {
                signers: vec![
                    Signer {
                        id: "signer3".to_string(),
                        public_key: vec![7, 8, 9],
                    },
                    Signer {
                        id: "signer4".to_string(),
                        public_key: vec![10, 11, 12],
                    },
                ],
                next_signers: vec!["signer5".to_string()],
                next_threshold: 1,
            };

            let event = IdEvent {
                id: "event_rotation".to_string(),
                previous: "prev_event".to_string(),
                timestamp: TIMESTAMP + 100,
                payload: IdEventKind::Rotation(rotation),
            };

            let persisted_event = PersistedIdEvent {
                id: "event_rotation".to_string(),
                payload: b"rotation".to_vec(),
                previous: "prev_event".to_string(),
                timestamp: TIMESTAMP + 100,
                proofs,
            };

            // Act
            let result = persisted_event.verify(&mut projection);

            // Assert
            assert!(result.is_ok());
            let updated_projection = result.unwrap();
            assert_eq!(updated_projection.event_id, "event_rotation");
            assert_eq!(updated_projection.signers.len(), 2);
            assert_eq!(updated_projection.signers[0].id, "signer3");
            assert_eq!(updated_projection.signers[1].id, "signer4");
            assert_eq!(updated_projection.next_signers.len(), 1);
            assert_eq!(updated_projection.next_signers[0], "signer5");
            assert_eq!(updated_projection.all_signers.len(), 5); // Original 2 + new 2 + next 1
        }

        #[test]
        fn test_verify_rotation_insufficient_threshold() {
            // Arrange
            let mut projection = IdProjection::new(
                "prev_event".to_string(),
                3,
                vec![
                    Signer {
                        id: "signer1".to_string(),
                        public_key: vec![1, 2, 3],
                    },
                    Signer {
                        id: "signer2".to_string(),
                        public_key: vec![4, 5, 6],
                    },
                    Signer {
                        id: "signer3".to_string(),
                        public_key: vec![7, 8, 9],
                    },
                ],
            );
            projection.next_signers = vec!["signer4".to_string()];
            projection.threshold = 3;

            let proofs = vec![Proof {
                id: "signer4".to_string(),
                pk: vec![10, 11, 12],
                sig: b"valid".to_vec(),
            }];

            let rotation = IdRotation {
                signers: vec![
                    Signer {
                        id: "signer4".to_string(),
                        public_key: vec![10, 11, 12],
                    },
                    Signer {
                        id: "signer5".to_string(),
                        public_key: vec![13, 14, 15],
                    },
                ],
                next_signers: vec!["signer6".to_string()],
                next_threshold: 2, // Insufficient threshold
            };

            let persisted_event = PersistedIdEvent {
                id: "event_rotation".to_string(),
                payload: b"rotation".to_vec(),
                previous: "prev_event".to_string(),
                timestamp: TIMESTAMP + 100,
                proofs,
            };

            // Act
            let result = persisted_event.verify(&mut projection);

            // Assert
            assert_eq!(result, Err(IdEventError::ThresholdNotMatch));
        }

        #[test]
        fn test_verify_rotation_duplicate_signers() {
            // Arrange
            let mut projection = IdProjection::new(
                "prev_event".to_string(),
                1,
                vec![Signer {
                    id: "signer1".to_string(),
                    public_key: vec![1, 2, 3],
                }],
            );
            projection.next_signers = vec!["signer3".to_string()];
            projection.threshold = 1;

            let proofs = vec![Proof {
                id: "signer3".to_string(),
                pk: vec![7, 8, 9],
                sig: b"valid".to_vec(),
            }];

            let rotation = IdRotation {
                signers: vec![
                    Signer {
                        id: "signer3".to_string(),
                        public_key: vec![7, 8, 9],
                    },
                    Signer {
                        id: "signer3".to_string(), // Duplicate
                        public_key: vec![7, 8, 9],
                    },
                ],
                next_signers: vec!["signer4".to_string()],
                next_threshold: 1,
            };

            let persisted_event = PersistedIdEvent {
                id: "event_rotation".to_string(),
                payload: b"rotation".to_vec(),
                previous: "prev_event".to_string(),
                timestamp: TIMESTAMP + 100,
                proofs,
            };

            // Act
            let result = persisted_event.verify(&mut projection);

            // Assert
            assert_eq!(
                result,
                Err(IdEventError::InvalidSigner(IdError {
                    id: "signer3".to_string(),
                    reason: "duplicate-signer".to_string(),
                }))
            );
        }

        #[test]
        fn test_verify_delegation_success() {
            // Arrange
            let mut projection = IdProjection::new(
                "prev_event".to_string(),
                1,
                vec![Signer {
                    id: "signer1".to_string(),
                    public_key: vec![1, 2, 3],
                }],
            );
            projection.next_signers = vec!["signer2".to_string()];

            let proofs = vec![Proof {
                id: "signer2".to_string(),
                pk: vec![4, 5, 6],
                sig: b"valid".to_vec(),
            }];

            let persisted_event = PersistedIdEvent {
                id: "event_delegation".to_string(),
                payload: b"delegation".to_vec(),
                previous: "prev_event".to_string(),
                timestamp: TIMESTAMP + 100,
                proofs,
            };

            // Act
            let result = persisted_event.verify(&mut projection);

            // Assert
            assert!(result.is_ok());
            let updated_projection = result.unwrap();
            assert_eq!(updated_projection.event_id, "event_delegation");
            assert_eq!(updated_projection.delegate_id, Some("new_id".to_string()));
        }

        #[test]
        fn test_verify_delegation_signer_not_authorized() {
            // Arrange
            let mut projection = IdProjection::new(
                "prev_event".to_string(),
                1,
                vec![Signer {
                    id: "signer1".to_string(),
                    public_key: vec![1, 2, 3],
                }],
            );
            projection.next_signers = vec!["signer2".to_string()];

            let proofs = vec![Proof {
                id: "signer3".to_string(), // Not authorized
                pk: vec![7, 8, 9],
                sig: b"valid".to_vec(),
            }];

            let persisted_event = PersistedIdEvent {
                id: "event_delegation".to_string(),
                payload: b"delegation".to_vec(),
                previous: "prev_event".to_string(),
                timestamp: TIMESTAMP + 100,
                proofs,
            };

            // Act
            let result = persisted_event.verify(&mut projection);

            // Assert
            assert_eq!(
                result,
                Err(IdEventError::InvalidProof(IdError {
                    id: "signer3".to_string(),
                    reason: "signer-not-authorized".to_string(),
                }))
            );
        }

        #[test]
        fn test_verify_delegation_invalid_delegated_id() {
            // Arrange
            let mut projection = IdProjection::new(
                "prev_event".to_string(),
                1,
                vec![Signer {
                    id: "signer1".to_string(),
                    public_key: vec![1, 2, 3],
                }],
            );
            projection.next_signers = vec!["signer2".to_string()];

            let proofs = vec![Proof {
                id: "signer2".to_string(),
                pk: vec![4, 5, 6],
                sig: b"valid".to_vec(),
            }];

            let persisted_event = PersistedIdEvent {
                id: "event_delegation_invalid".to_string(), // Invalid kind
                payload: b"delegation_invalid".to_vec(),    // Invalid payload
                previous: "prev_event".to_string(),
                timestamp: TIMESTAMP + 100,
                proofs,
            };

            // Act
            let result = persisted_event.verify(&mut projection);

            // Assert
            assert_eq!(
                result,
                Err(IdEventError::Other("invalid-delegated-id".to_string()))
            );
        }

        #[test]
        fn test_verify_invalid_event_kind() {
            // Arrange
            let mut projection = IdProjection::new(
                "prev_event".to_string(),
                1,
                vec![Signer {
                    id: "signer1".to_string(),
                    public_key: vec![1, 2, 3],
                }],
            );

            let proofs = vec![Proof {
                id: "signer1".to_string(),
                pk: vec![1, 2, 3],
                sig: b"valid".to_vec(),
            }];

            let persisted_event = PersistedIdEvent {
                id: "event_invalid_kind".to_string(),
                payload: b"unknown_kind".to_vec(), // Unknown payload
                previous: "prev_event".to_string(),
                timestamp: TIMESTAMP + 100,
                proofs,
            };

            // Act
            let result = persisted_event.verify(&mut projection);

            // Assert
            assert_eq!(result, Err(IdEventError::InvalidPayload));
        }

        #[test]
        fn test_verify_invalid_event_id_kind() {
            // Arrange
            let mut projection = IdProjection::new(
                "prev_event".to_string(),
                1,
                vec![Signer {
                    id: "signer1".to_string(),
                    public_key: vec![1, 2, 3],
                }],
            );

            let proofs = vec![Proof {
                id: "signer1".to_string(),
                pk: vec![1, 2, 3],
                sig: b"valid".to_vec(),
            }];

            let persisted_event = PersistedIdEvent {
                id: "invalid_kind_id".to_string(), // Assume this results in kind not "event"
                payload: b"interaction".to_vec(),
                previous: "prev_event".to_string(),
                timestamp: TIMESTAMP + 100,
                proofs,
            };

            // Act
            let result = persisted_event.verify(&mut projection);

            // Assert
            assert_eq!(
                result,
                Err(IdEventError::InvalidEventId("invalid_kind_id".to_string()))
            );
        }
    */
}
*/
