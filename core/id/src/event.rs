use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
use cid::Cid;
use idp2p_common::{cbor, cid::CidExt, verifying::ed25519::verify};
use serde::{Deserialize, Serialize};

use crate::{action::IdActionKind, config::IdConfig, signer::IdSigner, IdSnapshot, PersistedIdProof};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedIdEvent {
    pub id: Cid,
    pub payload: Vec<u8>,
    pub proofs: Vec<PersistedIdProof>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdEvent {
    pub timestamp: DateTime<Utc>,
    pub previous: Cid,
    pub signers: Vec<Cid>,
    pub payload: IdEventPayload,
    pub next_signers: Vec<IdSigner>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdEventPayload {
    Action(Vec<IdActionKind>),
    Config(IdConfig),
    Cancel(Cid),
}

impl IdEvent {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let event = cbor::decode(bytes)?;
        Ok(event)
    }

    pub fn validate(&self) -> Result<()> {
        let total_values: u16 = self.next_signers.iter().map(|x| x.value as u16).sum();
        /*if let Some(config) = &self.config  {
            if config.quorum == 0 {
                bail!("The quorum must be greater than 0.");
            }
            if total_values < config.quorum {
                bail!("The quorum must be less than or equal to the total values of signers.");
            }
        }*/

        Ok(())
    }
}

pub fn verify_event(snapshot: IdSnapshot, pevent: PersistedIdEvent) -> anyhow::Result<IdSnapshot> {
    /*let event_id = Cid::from_bytes(&pevent.id)?;
    event_id.ensure(pevent.payload.as_slice())?;
    let mut signers: Vec<IdSigner> = vec![];
    for proof in pevent.proofs {
        let signer_id = Cid::from_bytes(&proof.id)?;
        signer_id.ensure(&proof.pk)?;
        if let Some(signer) = snapshot.next_signers.iter().find(|x| x.id == signer_id.to_bytes()){
            verify(&proof.pk, &pevent.payload, &proof.sig)?;
            signers.push(signer.to_owned());
        }else{
            anyhow::bail!("invalid signer")
        }
    }

    let event = IdEvent::from_bytes(&pevent.payload)?;
    event.validate()?;
    let mut snapshot = snapshot;

    if event.previous != Cid::from_bytes(&snapshot.event_id)? {
        anyhow::bail!("invalid previous")
    }

    // Check signer quorum
    match event.payload {
        IdEventPayload::ChangeState(state) => {
            if snapshot.used_states.contains(&state.to_bytes()) {
                anyhow::bail!("duplicated state")
            }
            snapshot.state = state.to_bytes();
            snapshot.used_states.push(state.to_bytes());
        },
        IdEventPayload::ChangeConfig(id_config) => {
            id_config.validate()?;
            snapshot.config = id_config;
        },
        IdEventPayload::RevokeEvent => todo!(),
    }
    for signer in signers.iter() {
        snapshot.used_signers.push(signer.id.clone());
    }
    snapshot.next_signers = event.next_signers;
    Ok(snapshot)*/
    todo!()
}


mod tests {
    use cid::{multihash::Multihash, Cid};

    use idp2p_common::ED_CODE;

    use super::*;

    fn create_signer() -> Result<IdSigner> {
        let multihash = Multihash::<64>::wrap(0x12, b"test").unwrap(); // Create a Cid with codec ED_CODE
        let cid = Cid::new_v1(ED_CODE, multihash);
        // Value greater than 0
        let value = 1;
        // Attempt to create a new IdSigner
        let signer = IdSigner{value, id: cid };
        Ok(signer)
    }

    #[test]
    fn event_test() -> Result<()> {
        let signer = create_signer()?;
        let previous = Cid::new_v1(ED_CODE, Multihash::<64>::wrap(0x12, b"test").unwrap());
        let next_signers = vec![signer.clone()];
      
        //assert_eq!(event.version, "0.1.0");
        Ok(())
    }
}
