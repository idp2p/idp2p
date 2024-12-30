use std::str::FromStr;

use idp2p_common::{cbor, ed25519::verify, said::Said};

use crate::{
    idp2p::id::{
        error::IdError,
        types::{IdActionKind::*, IdEvent, IdEventKind::*},
    },
    validation::SaidValidator,
    IdEventError, IdView, PersistedIdEvent, TIMESTAMP, VERSION,
};

impl PersistedIdEvent {
    pub(crate) fn verify(&self, view: &mut IdView) -> Result<IdView, IdEventError> {
        let event: IdEvent = self.try_into()?;

        // Timestamp check
        //
        if event.timestamp < TIMESTAMP {
            return Err(IdEventError::InvalidTimestamp);
        }

        // Previous event check
        //
        if event.previous != view.event_id {
            return Err(IdEventError::PreviousNotMatch);
        }

        // Proof verification
        //
        let mut signers = vec![];
        for proof in &self.proofs {
            let sid = Said::from_str(proof.id.as_str()).map_err(|e| {
                IdEventError::InvalidProof(IdError {
                    id: proof.id.clone(),
                    reason: e.to_string(),
                })
            })?;
            sid.validate(&proof.pk).map_err(|e| {
                IdEventError::InvalidProof(IdError {
                    id: proof.id.clone(),
                    reason: e.to_string(),
                })
            })?;
            if signers.contains(&proof.id) {
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
            signers.push(proof.id.clone());
        }

        match event.payload {
            Interaction(actions) => {
                if (signers.len() as u8) < view.threshold {
                    return Err(IdEventError::LackOfMinProofs);
                }
                for signer in &signers {
                    if !view.signers.iter().any(|s| s.id == *signer) {
                        return Err(IdEventError::InvalidProof(IdError {
                            id: signer.clone(),
                            reason: "signer-not-found".to_string(),
                        }));
                    }
                }
                // Check signers and threshold
                for action in actions {
                    match action {
                        CreatePeer(_) => todo!(),
                        RevokePeer(_) => todo!(),
                        CreateMediator(_) => todo!(),
                        RevokeMediator(_) => todo!(),
                        CreateClaim(id_claim) => {
                            view.claims.push(id_claim.to_owned());
                        }
                        RevokeClaim(id) => {
                            if let Some(claim) = view.claims.iter().find(|c| c.id == id) {
                                view.claims
                                    .remove(view.claims.iter().position(|c| c.id == id).unwrap());
                            }
                            //return Err(IdEventError::ClaimNotFound);
                        }
                    }
                }
            }
            Rotation(id_rotation) => {
                // Check signers and threshold
                for signer in &id_rotation.signers {
                    /*let signer_said = Said::from_str(signer.id.as_str())?;
                    signer_said.validate(&signer.public_key)?;
                    signer_said.ensure_signer()?;*/
                }
            }
            Delegation(new_id) => {
                //
            }
        }

        Ok(view.to_owned())
    }
}

impl TryFrom<&PersistedIdEvent> for IdEvent {
    type Error = IdEventError;

    fn try_from(value: &PersistedIdEvent) -> Result<Self, Self::Error> {
        let said: Said = Said::from_str(value.id.as_str())
            .map_err(|e| IdEventError::InvalidEventId(e.to_string()))?;
        if said.version != VERSION {
            return Err(IdEventError::InvalidVersion);
        }
        said.validate(&value.payload)
            .map_err(|e| IdEventError::InvalidEventId(e.to_string()))?;
        said.ensure_event()
            .map_err(|_| IdEventError::PayloadAndIdNotMatch)?;

        let event: IdEvent =
            cbor::decode(&value.payload).map_err(|_| IdEventError::InvalidPayload)?;
        Ok(event)
    }
}
