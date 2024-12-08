use anyhow::Result;
use cid::Cid;
use idp2p_common::cid::CidExt;

use crate::{inception::IdInception, IdView, PersistedIdEvent, PersistedIdInception};

pub fn verify_inception(pid: PersistedIdInception) -> Result<IdView> {
    let inception = IdInception::from_bytes(&pid.payload)?;
    Cid::from_bytes(&pid.id)?.ensure(pid.payload.as_slice())?;
    inception.validate()?;
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

pub fn verify_event(view: IdView, event: PersistedIdEvent) -> Result<IdView> {
    //verify_event(snapshot, event).map_err(|e| e.to_string())
    todo!()
}
