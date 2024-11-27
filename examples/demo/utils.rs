use anyhow::Result;
use cid::Cid;
use ed25519_dalek::SigningKey;
use idp2p_common::{cbor, cid::CidExt};
use idp2p_id::{inception::IdInception, model::{id::PersistedId, inception::PersistedIdInception}};
use libp2p::PeerId;
use rand::rngs::OsRng;

pub fn generate_id(mediator: &PeerId) -> Result<PersistedId> {
    let mut csprng = OsRng;
    let signing_key: SigningKey = SigningKey::generate(&mut csprng);
    let inception = IdInception::from_signer(&signing_key.to_bytes(), &mediator.to_string())?;
    let payload = cbor::encode(&inception)?;
    let cid = Cid::create(0x01, payload.as_slice())?;
    let persisted_id = PersistedId {
        id: cid.clone(),
        incepiton: PersistedIdInception {
            id: cid,
            payload: payload,
        },
        events: vec![],
    };
    Ok(persisted_id)
}
