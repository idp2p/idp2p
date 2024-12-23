use alloc::vec::Vec;
use cid::Cid;
use idp2p_common::{cbor, cid::CidExt, verifying::ed25519::verify, ED_CODE};

use crate::{
    internal::IdEvent, internal::IdEventPayload::*, internal::IdInception, IdEventError,
    IdInceptionError, IdView, PersistedIdEvent, PersistedIdInception,
};

pub fn verify_inception(pid: PersistedIdInception) -> Result<IdView, IdInceptionError> {
    let id = Cid::try_from(pid.id.as_str()).map_err(|_| IdInceptionError::InvalidId)?;
    id.ensure(pid.payload.as_slice())
        .map_err(|e| IdInceptionError::PayloadAndIdNotMatch(e.to_string()))?;
    let inception: IdInception =
        cbor::decode(&pid.payload).map_err(|_| IdInceptionError::InvalidPayload)?;
    let total_signers = inception.next_signers.len() as u8;
    if total_signers < inception.threshold {
        return Err(IdInceptionError::TotalNextSignersNotMatch(
            inception.threshold,
        ));
    }
    for next_signer in &inception.next_signers {
        let next_signer_cid = Cid::try_from(next_signer.as_str())
            .map_err(|_| IdInceptionError::InvalidNextSignerCodec(next_signer.clone()))?;
        if next_signer_cid.codec() != ED_CODE {
            return Err(IdInceptionError::InvalidNextSignerCodec(
                next_signer.clone(),
            ));
        }
    }

    let id_str = format!("/p2p/id/{}", id.to_string());
    let id_view = IdView {
        id: id_str,
        event_id: pid.id.clone(),
        event_timestamp: inception.timestamp.to_string(),
        next_signers: inception.next_signers.clone(),
        signers: todo!(),
        mediators: todo!(),
        authentication: todo!(),
        key_agreement: todo!(),
        assertation: todo!(),
        threshold: todo!(),
        claims: todo!(),
    };

    Ok(id_view)
}

pub fn verify_event(view: IdView, pevent: PersistedIdEvent) -> Result<IdView, IdEventError> {
    let event_id = Cid::try_from(pevent.id.as_str()).map_err(|_| IdEventError::InvalidEventId)?;
    event_id
        .ensure(pevent.payload.as_slice())
        .map_err(|e| IdEventError::PayloadAndIdNotMatch(e.to_string()))?;
    let event: IdEvent = cbor::decode(&pevent.payload).map_err(|_| IdEventError::InvalidPayload)?;
    if event.previous != view.event_id {
        return Err(IdEventError::PreviousNotMatch(event.previous));
    }

    for proof in pevent.proofs {
        let signer_id = Cid::try_from(proof.id.as_str())
            .map_err(|_| IdEventError::InvalidNextSignerCodec(proof.id.clone()))?;
        signer_id
            .ensure(&proof.pk)
            .map_err(|_| IdEventError::SignerAndIdNotMatch(proof.id.clone()))?;
        if !view.next_signers.iter().any(|x| x == proof.id.as_str()) {
            return Err(IdEventError::InvalidNextSigner(proof.id.clone()));
        }
        verify(&proof.pk, &pevent.payload, &proof.sig)
            .map_err(|_| IdEventError::InvalidSignature(proof.id.clone()))?;
    }
    let mut view = view;
    /*match event.payload {
        Rotation(rotation) => {
            if let Some(state) = rotation.state {
                view.state = state.to_string();
                view.all_states.push(state.to_string());
            }
            for med in rotation.mediators {
                match med {
                    Add(kid) => view.mediators.push(kid.to_bytes()),
                    Remove(kid) => view.mediators.retain(|x| *x != kid.to_bytes()),
                }
            }
        }
        Recovery(config) => {
            if let Some(config) = config {
                view.config = config;
            }
        }
    }
    view.event_id = event_id.to_bytes();
    view.event_timestamp = event.timestamp.to_string();
    view.next_signers = event.next_signers.iter().map(|x| x.to_bytes()).collect();*/

    Ok(view)
}
