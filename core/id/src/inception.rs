use crate::{IdConfig, IdSigner, IdSnapshot, PersistedIdInception, VERSION};

use anyhow::{bail, Result};
use chrono::prelude::*;
use cid::Cid;
use idp2p_common::{cbor, cid::CidExt};
use semver::Version;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdInception {
    pub version: Version,
    pub state: Cid,
    pub config: IdConfig,
    pub timestamp: DateTime<Utc>,
    pub next_signers: Vec<IdSigner>,
}

impl IdInception {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let inception = cbor::decode(bytes)?;
        Ok(inception)
    }

    fn validate(&self) -> Result<()> {
        if &self.version.to_string() != VERSION {
            bail!("The version");
        }
        let total_values: u16 = self.next_signers.iter().map(|x| x.value as u16).sum();

        if total_values < self.config.change_config_quorum {
            bail!("The quorum must be less than or equal to the total values of signers.");
        }
        self.config.validate()?;
        for signer in &self.next_signers {
            signer.validate()?;
        }
        Ok(())
    }
}

pub fn verify_inception(persisted_inception: PersistedIdInception) -> anyhow::Result<IdSnapshot> {
    let inception = IdInception::from_bytes(&persisted_inception.payload)?;
    let cid = Cid::from_bytes(persisted_inception.id.as_slice())?;
    cid.ensure(persisted_inception.payload.as_slice())?;
    inception.validate()?;
    let signer_ids: Vec<Vec<u8>> = inception.next_signers.iter().map(|s| s.id.clone()).collect();
    let id_snapshot = IdSnapshot {
        id: persisted_inception.id.clone(),
        event_id: persisted_inception.id.clone(),
        config: inception.config,
        state: inception.state.to_bytes(),
        event_timestamp: inception.timestamp.to_string(),
        next_signers: inception.next_signers,
        used_signers: signer_ids,
        used_states: vec![inception.state.to_bytes()],
    };
    Ok(id_snapshot)
}

mod tests {
    use cid::{multihash::Multihash, Cid};
    use idp2p_common::ED_CODE;

    use super::*;

    fn new(config: IdConfig, state: Cid, next_signers: Vec<IdSigner>) -> Result<IdInception> {
        let ver = semver::Version::parse("0.0.1").map_err(anyhow::Error::msg)?;
        let inception = IdInception {
            version: ver,
            config: config,
            timestamp: Utc::now(),
            state: state,
            next_signers: next_signers,
        };
        Ok(inception)
    }

    fn create_signer() -> Result<IdSigner> {
        let multihash = Multihash::<64>::wrap(0x12, b"test").unwrap(); // Create a Cid with codec ED_CODE
        let cid = Cid::new_v1(ED_CODE, multihash);
        // Value greater than 0
        let value = 1;
        // Attempt to create a new IdSigner
        let signer = IdSigner {
            value,
            id: cid.to_bytes(),
        };
        Ok(signer)
    }

    #[test]
    fn incept_test() -> Result<()> {
        let signer = create_signer()?;
        let next_signers = vec![signer];
        let inception = new(IdConfig::default(), Cid::default(), next_signers)?;
        assert_eq!(inception.version, Version::parse("0.1.0").unwrap());
        Ok(())
    }
}
