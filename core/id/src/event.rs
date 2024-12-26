use cid::Cid;
use idp2p_common::{
    cbor, cid::CidExt, utils::parse_id, verifying::ed25519::verify, ED_CODE,
};


use crate::{
    idp2p::id::{error::IdInceptionErrorKind, types::{IdEvent, IdInception, IdActionKind::*, IdEventKind::*}},
    IdEventError, IdInceptionError, IdView, PersistedIdEvent, PersistedIdInception,
};

pub fn verify_inception(pid: PersistedIdInception) -> Result<IdView, IdInceptionError> {
    // Decode 
    // 
    let inception: IdInception = pid.try_into()?;
    // Timestamp check
    // 
    
    // Signer threshold, codec, public bytes check 
    // 
    let total_next_signers = inception.next_signers.len() as u8;
    if total_next_signers < inception.next_threshold {
        todo!("")
    }

    for next_signer in &inception.next_signers {
        let next_signer_cid = Cid::try_from(next_signer.as_str())?;
        if next_signer_cid.codec() != ED_CODE {
            todo!("")
        }
    }

    // Next Signer threshold, codec check 
    // 
    let total_signers = inception.signers.len() as u8;
    if total_signers < inception.threshold {
        todo!("")
    }

    for signer in &inception.signers {
        let signer_cid = Cid::try_from(signer.id.as_str())?;
        if signer_cid.codec() != ED_CODE {
            todo!("codec error")
        }
        todo!("check public key")
    }

    // Claims check 
    // 
    for action in inception.actions {
        match action {
            CreateClaim(id_claim) => todo!(),
            RevokeClaim(_) => todo!("Error"),
        }
    }

    let id_view = IdView {
        id: pid.id,
        event_id: pid.id.clone(),
        event_timestamp: inception.timestamp,
        next_signers: inception.next_signers.clone(),
        signers: todo!(),
        threshold: todo!(),
        claims: todo!(),
    };

    Ok(id_view)
}

pub fn verify_event(view: IdView, pevent: PersistedIdEvent) -> Result<IdView, IdEventError> {
    let event: IdEvent = pevent.try_into()?;
    if event.previous != view.event_id {
        //return Err(IdEventError::PreviousNotMatch(event.previous));
    }

    // Timestamp check
    // Verify proofs and check signer if it exists in event.signers

    match event.payload {
        Interaction(actions) => {
            // Check signers and threshold
            for action in actions {
                match action {
                    CreateClaim(id_claim) => todo!(),
                    RevokeClaim(id) => todo!(),
                }
            }
        },
        Rotation(id_rotation) => {

        },
        Delegation(_) => {

        },
    }

    /*for proof in pevent.proofs {
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
    }*/
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

impl TryFrom<PersistedIdInception> for IdInception {
    type Error = IdInceptionError;

    fn try_from(value: PersistedIdInception) -> Result<Self, Self::Error> {
        let (_, cid) =
            parse_id_with_version("id", &value.id).map_err(|_| Self::Error::InvalidId)?;
        cid.ensure(&value.payload)
            .map_err(|_| Self::Error::InvalidId)?;
        let inception: IdInception =
            cbor::decode(&value.payload).map_err(|_| Self::Error::InvalidId)?;
        Ok(inception)
    }
}

impl TryFrom<PersistedIdEvent> for IdEvent {
    type Error;

    fn try_from(value: PersistedIdEvent) -> Result<Self, Self::Error> {
        let event_id = Cid::try_from(pevent.id.as_str()).map_err(|_| IdEventError::InvalidEventId)?;
        event_id
            .ensure(pevent.payload.as_slice())
            .map_err(|e| IdEventError::PayloadAndIdNotMatch(e.to_string()))?;
        let event: IdEvent = cbor::decode(&pevent.payload).map_err(|_| IdEventError::InvalidPayload)?;
        todo!()
    }
}
