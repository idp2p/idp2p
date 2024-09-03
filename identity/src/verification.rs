use crate::{
    event::{IdEvent, IdInception, IdActionKind::*},
    exports::idp2p::id::verification::{IdState, PersistedIdEvent, PersistedIdInception},
};
use cid::Cid;
use idp2p_core::{cid::CidExt, verifying::Ed25519PublicKey};

pub fn verify_inception(persisted_inception: PersistedIdInception) -> anyhow::Result<IdState> {
    let inception = IdInception::from_bytes(&persisted_inception.payload)?;
    let cid = Cid::from_bytes(persisted_inception.id.as_slice())?;
    cid.ensure(persisted_inception.payload.as_slice())?;
    let mut id_state = IdState {
        id: persisted_inception.id.clone(),
        event_id: persisted_inception.id.clone(),
        signer: inception.signer,
        recovery: inception.recovery,
        state: None,
        assertions: vec![],
        authentications: vec![],
        agreements: vec![],
        used: vec![],
    };
    for a in inception.actions.iter() {
        match a {
            UpdateState(_) => todo!(),
            AddAssertionKey(_) => todo!(),
            RemoveAssertionKey(_) => todo!(),
            AddAuthenticationKey(_) => todo!(),
            RemoveAuthenticationKey(_) => todo!(),
            AddAgreementKey(_) => todo!(),
            RemoveAgreementKey(_) => todo!(),
        }
    }
    Ok(IdState {
        id: persisted_inception.id.clone(),
        event_id: persisted_inception.id.clone(),
        signer: inception.signer,
        recovery: inception.recovery,
        // find update state event and set state
        state: inception.events.iter().filter(|e| matches!(e.payload, IdEventKind::UpdateState(_))).collect(),
        assertions: todo!(),
        authentications: todo!(),
        agreements: todo!(),
        used: todo!(),
    })
}

pub fn verify_event(state: IdState, event: IdEvent) -> anyhow::Result<IdState> {
    let payload = event.to_bytes()?;
    let mut state = state;
    let event_id = Cid::from_bytes(&event.id)?;
    event_id.ensure(payload.as_slice())?;

    if event.previous != state.latest_event_id {
        anyhow::bail!("invalid previous")
    }

    if state.events.contains(&event_id.to_bytes()) {
        anyhow::bail!("duplicated event id")
    }

    if event.proofs.len() < state.latest_m_of_n.m as usize {
        anyhow::bail!("insufficient signatures")
    }

    for proof in event.proofs {
        let signer_id = Cid::from_bytes(&proof.signer_id)?;
        signer_id.ensure(&proof.signer_pub)?;
        if !state.signers.contains(&proof.signer_id) {
            anyhow::bail!("invalid signer")
        }
        let pub_key = Ed25519PublicKey::from_bytes(&proof.signer_pub)?;
        let signature = proof
            .signature
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid signature bytes"))?;
        pub_key.verify(&event.id, signature)?;
    }
    if state.states.contains(&event.state) {
        anyhow::bail!("duplicated state")
    }
    state.latest_state = event.state.clone();
    state.states.push(event.state);
    if let Some(m_of_n) = event.m_of_n {
        state.latest_m_of_n = m_of_n;
    }

    if state.latest_signers.len() < state.latest_m_of_n.n as usize {
        anyhow::bail!("insufficient new signers")
    }

    state.events.push(event_id.to_bytes());
    state.latest_event_id = event_id.to_bytes();
    Ok(state)
}
