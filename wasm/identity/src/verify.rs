use cid::Cid;
use crate::{IdEvent, IdInception, IdState};
use idp2p_core::{cid::CidExt, verifying::Ed25519PublicKey};

pub fn verify_inception(inception: IdInception) -> anyhow::Result<IdState> {
    let payload = inception.to_bytes()?;
    let cid = Cid::from_bytes(inception.id.as_slice())?;
    cid.ensure(payload.as_slice())?;
    if inception.m_of_n.m >= inception.m_of_n.n {
        anyhow::bail!("invalid m of n")
    }
    if inception.signers.len() < inception.m_of_n.n as usize {
        anyhow::bail!("not enough signers")
    }
    Ok(IdState {
        id: inception.id.clone(),
        latest_m_of_n: inception.m_of_n,
        latest_signers: inception.signers.clone(),
        latest_event_id: inception.id.clone(),
        latest_state: inception.state.clone(),
        signers: inception.signers,
        states: vec![inception.state],
        events: vec![inception.id],
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
