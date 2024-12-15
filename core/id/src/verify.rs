use cid::Cid;
use idp2p_common::{cbor, cid::CidExt, verifying::ed25519::verify, ED_CODE};

use crate::{
    internal::IdEvent, internal::IdEventPayload::*, internal::IdInception,
    internal::IdMediatorAction::*, IdEventError, IdInceptionError, IdView, PersistedIdEvent,
    PersistedIdInception,
};

pub fn verify_inception(pid: PersistedIdInception) -> Result<IdView, IdInceptionError> {
    let id = Cid::from_bytes(&pid.id).map_err(|_| IdInceptionError::InvalidId)?;
    id.ensure(pid.payload.as_slice())
        .map_err(|e| IdInceptionError::PayloadAndIdNotMatch(e.to_string()))?;
    let inception: IdInception =
        cbor::decode(&pid.payload).map_err(|_| IdInceptionError::InvalidPayload)?;
    let total_signers: u16 = inception.next_signers.len() as u16;
    if total_signers != inception.config.multisig.total_signers() {
        return Err(IdInceptionError::TotalNextSignersNotMatch(total_signers));
    }
    for signer in &inception.next_signers {
        if signer.codec() != ED_CODE {
            return Err(IdInceptionError::InvalidNextSignerCodec(signer.to_bytes()));
        }
    }
    let all_signers: Vec<Vec<u8>> = inception
        .next_signers
        .iter()
        .map(|s| s.to_bytes())
        .collect();
    let id_view = IdView {
        id: pid.id.clone(),
        config: inception.config,
        state: inception.state.to_bytes(),
        event_id: pid.id.clone(),
        event_timestamp: inception.timestamp.to_string(),
        mediators: inception.mediators.iter().map(|s| s.to_bytes()).collect(),
        next_signers: all_signers.clone(),
        all_signers: all_signers,
        all_states: vec![inception.state.to_bytes()],
    };

    Ok(id_view)
}

pub fn verify_event(view: IdView, pevent: PersistedIdEvent) -> Result<IdView, IdEventError> {
    let event_id = Cid::from_bytes(&pevent.id).map_err(|_| IdEventError::InvalidEventId)?;
    event_id
        .ensure(pevent.payload.as_slice())
        .map_err(|e| IdEventError::PayloadAndIdNotMatch(e.to_string()))?;
    let event: IdEvent = cbor::decode(&pevent.payload).map_err(|_| IdEventError::InvalidPayload)?;
    if event.previous
        != Cid::from_bytes(&view.event_id).map_err(|_| IdEventError::InvalidEventId)?
    {
        return Err(IdEventError::PreviousNotMatch(event.previous.to_bytes()));
    }
    for proof in pevent.proofs {
        let signer_id = Cid::from_bytes(&proof.id)
            .map_err(|_| IdEventError::InvalidNextSignerCodec(proof.id.clone()))?;
        signer_id
            .ensure(&proof.pk)
            .map_err(|_| IdEventError::SignerAndIdNotMatch(signer_id.to_bytes()))?;
        if !view
            .next_signers
            .iter()
            .any(|x| x.to_vec() == signer_id.to_bytes())
        {
            return Err(IdEventError::InvalidNextSigner(signer_id.to_bytes()));
        }
        verify(&proof.pk, &pevent.payload, &proof.sig)
            .map_err(|_| IdEventError::InvalidSignature(signer_id.to_bytes()))?;
    }
    let mut view = view;
    match event.payload {
        Action(action) => {
            if let Some(cid) = action.state {
                view.state = cid.to_bytes();
                view.all_states.push(cid.to_bytes());
            }
            for med in action.mediators {
                match med {
                    Add(cid) => view.mediators.push(cid.to_bytes()),
                    Remove(cid) => view.mediators.retain(|x| *x != cid.to_bytes()),
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
    view.next_signers = event.next_signers.iter().map(|x| x.to_bytes()).collect();

    Ok(view)
}
