use cid::Cid;
use idp2p_common::{cbor, cid::CidExt, verifying::ed25519::verify, ED_CODE};

use crate::{
    event::IdActionKind::*, event::IdEvent, event::IdEventPayload::*, inception::IdInception,
    IdEventError, IdInceptionError, IdView, PersistedIdEvent, PersistedIdInception,
};

pub fn verify_inception(pid: PersistedIdInception) -> Result<IdView, IdInceptionError> {
    let id = Cid::from_bytes(&pid.id).map_err(|_| IdInceptionError::InvalidId)?;
    id.ensure(pid.payload.as_slice())
        .map_err(|e| IdInceptionError::PayloadAndIdNotMatch(e.to_string()))?;
    let inception: IdInception =
        cbor::decode(&pid.payload).map_err(|_| IdInceptionError::InvalidPayload)?;
    let total_signers: u16 = inception.next_signers.len() as u16;
    if total_signers != inception.multisig.total_signers() {
        return Err(IdInceptionError::TotalNextSignersNotMatch(total_signers));
    }
    for signer in &inception.next_signers {
        if signer.codec() != ED_CODE {
            return Err(IdInceptionError::InvalidNextSignerCodec(signer.to_bytes()));
        }
    }

    let id_view = IdView {
        id: pid.id.clone(),
        multisig: inception.multisig,
        state: inception.state.to_bytes(),
        event_id: pid.id.clone(),
        event_timestamp: inception.timestamp.to_string(),
        mediators: inception.mediators.iter().map(|s| s.to_bytes()).collect(),
        next_signers: inception
            .next_signers
            .iter()
            .map(|s| s.to_bytes())
            .collect(),
    };

    Ok(id_view)
}

pub fn verify_event(view: IdView, pevent: PersistedIdEvent) -> Result<IdView, IdEventError> {
    let event_id = Cid::from_bytes(&pevent.id).map_err(|_| IdEventError::InvalidId)?;
    event_id
        .ensure(pevent.payload.as_slice())
        .map_err(|e| IdEventError::PayloadAndIdNotMatch(e.to_string()))?;
    let event: IdEvent = cbor::decode(&pevent.payload).map_err(|_| IdEventError::InvalidPayload)?;
    if event.previous != Cid::from_bytes(&view.event_id).map_err(|_| IdEventError::InvalidId)? {
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
            return Err(IdEventError::InvalidId);
        }
        verify(&proof.pk, &pevent.payload, &proof.sig).map_err(|e| IdEventError::InvalidId)?;
    }
    let mut view = view;
    match event.payload {
        Action(actions) => {
            for action in actions {
                match action {
                    AddMediator(cid) => view.mediators.push(cid.to_bytes()),
                    RemoveMediator(cid) => view.mediators.retain(|x| *x != cid.to_bytes()),
                    UpdateState(cid) => view.state = cid.to_bytes(),
                }
            }
        }
        CancelEvent(event_id) => {

        }
        UpgradeId(id) => {

        }
    }
   
    view.next_signers = event.next_signers.iter().map(|x| x.to_bytes()).collect();
    /*

    // Check signer quorum
    match event.payload {
        IdEventPayload::ChangeState(state) => {
            if snapshot.used_states.contains(&state.to_bytes()) {
                anyhow::bail!("duplicated state")
            }
            snapshot.state = state.to_bytes();
            snapshot.used_states.push(state.to_bytes());
        }
        IdEventPayload::ChangeConfig(id_config) => {
            id_config.validate()?;
            snapshot.config = id_config;
        }
        IdEventPayload::RevokeEvent => todo!(),
    }
    for signer in signers.iter() {
        snapshot.used_signers.push(signer.id.clone());
    }
    snapshot.next_signers = event.next_signers;*/
    Ok(view)
}
