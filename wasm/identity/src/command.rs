use alloc::vec::Vec;
use cid::Cid;
use idp2p_core::cid::CidExt;
use idp2p_core::verifying::Ed25519PublicKey;

use crate::encoder::{event_to_bytes, inception_to_bytes};
use crate::idp2p::wasmid::model::{ IdKey, IdVersion};
use crate::{
    IdCreateEventResult, IdCreateResult, IdEvent, IdEventInput, IdInception, IdInceptionInput,
    BINARY_CODE,
};

pub fn create(input: IdInceptionInput) -> anyhow::Result<IdCreateResult> {
    let mut signers: Vec<Vec<u8>> = vec![];
    let mut signers_map: Vec<IdKey> = vec![];

    for signer in input.signers.iter() {
        let signer_pub = Ed25519PublicKey::from_bytes(&signer)?;
        let signer_id = signer_pub.to_id()?.to_bytes();
        signers.push(signer_id.clone());
        signers_map.push(IdKey {
            id: signer_id,
            pk: signer.clone(),
        });
    }

    let payload = inception_to_bytes(&IdVersion::new(), &input.state, &signers, &input.m_of_n)?;
    let id = Cid::create(BINARY_CODE, &payload)?;
    let inception = IdInception {
        id: id.to_bytes(),
        version: IdVersion::new(),
        state: input.state,
        signers,
        m_of_n: input.m_of_n,
    };
    Ok(IdCreateResult {
        inception: inception,
        signers: signers_map,
    })
}

pub fn create_event(input: IdEventInput) -> anyhow::Result<IdCreateEventResult> {
    let mut new_signers: Vec<Vec<u8>> = vec![];
    let mut new_signers_map: Vec<IdKey> = vec![];
    if let Some(signers) = input.new_signers {
        for signer in signers.iter() {
            let signer_pub = Ed25519PublicKey::from_bytes(&signer)?;
            let signer_id = signer_pub.to_id()?.to_bytes();
            new_signers.push(signer_id.clone());
            new_signers_map.push(IdKey {
                id: signer_id,
                pk: signer.clone(),
            });
        }
    }

    let new_signers = if !new_signers.is_empty() {
        Some(new_signers)
    } else {
        None
    };

    let new_signers_map = if !new_signers_map.is_empty() {
        Some(new_signers_map)
    } else {
        None
    };

    let payload = event_to_bytes(
        &IdVersion::new(),
        &input.state.latest_event_id,
        &input.new_state,
        &new_signers,
        &input.new_m_of_n,
    )?;
    let id = Cid::create(BINARY_CODE, &payload)?;
    let event = IdEvent {
        version: IdVersion::new(),
        id: id.to_bytes(),
        previous: input.state.latest_event_id,
        state: input.new_state,
        signers: new_signers,
        m_of_n: input.new_m_of_n,
        proofs: vec![],
    };
    Ok(IdCreateEventResult{
        event,
        signers: new_signers_map
    })
}
