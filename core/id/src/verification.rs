use cid::Cid;
use idp2p_common::{cid::CidExt, id::{event::IdEvent, inception::IdInception}};

use crate::{IdSigner, IdSnapshot, PersistedIdEvent, PersistedIdInception};

impl TryFrom<IdSigner> for idp2p_common::id::signer::IdSigner {
    type Error = anyhow::Error;

    fn try_from(signer: IdSigner) -> Result<Self, Self::Error> {
        Ok(idp2p_common::id::signer::IdSigner {
            id: Cid::from_bytes(&signer.id)?,
            value: signer.value,
        })
    }
}

impl TryFrom<idp2p_common::id::signer::IdSigner> for IdSigner {
    type Error = anyhow::Error;

    fn try_from(signer: idp2p_common::id::signer::IdSigner) -> Result<Self, Self::Error> {
        Ok(IdSigner {
            id: signer.id.to_bytes(),
            value: signer.value,
        })
    }
}

pub fn verify_inception(persisted_inception: PersistedIdInception) -> anyhow::Result<IdSnapshot> {
    let inception = IdInception::from_bytes(&persisted_inception.payload)?;
    let cid = Cid::from_bytes(persisted_inception.id.as_slice())?;
    let i = "".parse::<i32>();
    cid.ensure(persisted_inception.payload.as_slice())?;
    let next_signers: anyhow::Result<Vec<IdSigner>> = inception
        .next_signers
        .into_iter()
        .map(|x| x.try_into())
        .collect();

    let id_snapshot = IdSnapshot {
        id: persisted_inception.id.clone(),
        event_id: persisted_inception.id.clone(),
        state: inception.state.to_bytes(),
        event_timestamp: inception.timestamp.to_string(),
        next_quorum: inception.config.quorum,
        next_signers: next_signers?,
        used_signers: vec![],
        used_states: vec![],
        key_reuse: inception.config.key_reuse,
    };
    Ok(id_snapshot)
}

pub fn verify_event(state: IdSnapshot, event: PersistedIdEvent) -> anyhow::Result<IdSnapshot> {
    let event = IdEvent::from_bytes(&event.payload)?;
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
