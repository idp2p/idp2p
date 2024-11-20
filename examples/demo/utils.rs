use anyhow::Result;
use cid::Cid;
use idp2p_common::{cbor, cid::CidExt};
use idp2p_id::{
    inception::{IdInception, PersistedIdInception},
    PersistedId,
};
use libp2p::PeerId;

pub fn generate_id(mediator: PeerId) -> Result<PersistedId> {
    let inception = IdInception::from_signer(b"", &mediator.to_string())?;
    let payload = cbor::encode(&inception)?;
    let id = Cid::create(0x01, payload.as_slice())?;
    let persisted_id = PersistedId {
        id: id.clone(),
        incepiton: PersistedIdInception {
            id: id,
            payload: payload,
        },
        events: vec![]
    };
    Ok(persisted_id)
}
