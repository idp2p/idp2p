use anyhow::Result;
use chrono::{DateTime, Utc};
use cid::Cid;
use idp2p_common::cbor;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdActionKind {
    AddMediator(Cid),
    RemoveMediator(Cid),
    UpdateState(Cid),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdEventPayload {
    Action(Vec<IdActionKind>),
    CancelEvent(Cid),
    UpgradeId(Cid),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdEvent {
    pub timestamp: DateTime<Utc>,
    pub previous: Cid,
    pub signers: Vec<Cid>,
    pub payload: IdEventPayload,
    pub next_signers: Vec<Cid>,
}

impl IdEvent {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let event = cbor::decode(bytes)?;
        Ok(event)
    }

    pub fn validate(&self) -> Result<()> {
        for signer in &self.next_signers {
            signer.validate()?;
        }
        match &self.payload {
            IdEventPayload::UpgradeId(config) => config.validate()?,
            _ => {}
        }
        Ok(())
    }
}

mod tests {
    use cid::{multihash::Multihash, Cid};

    use idp2p_common::ED_CODE;

    use super::*;

    fn create_signer() -> Result<IdSigner> {
        let multihash = Multihash::<64>::wrap(0x12, b"test").unwrap(); // Create a Cid with codec ED_CODE
        let cid = Cid::new_v1(ED_CODE, multihash).to_bytes();
        // Value greater than 0
        let value = 1;
        // Attempt to create a new IdSigner
        let signer = IdSigner { value, id: cid };
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
