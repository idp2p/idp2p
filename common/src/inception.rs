use crate::signer::IdNextSigners;
use anyhow::Result;
use chrono::prelude::*;
use cid::Cid;
use semver::Version;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdInception {
    pub version: Version,
    pub timestamp: DateTime<Utc>,
    pub next_signers: IdNextSigners,
    pub state: Cid,
}

impl IdInception {
    pub fn new(version: &str, state: Cid, next_signers: IdNextSigners) -> Result<Self> {
        let ver = semver::Version::parse(version).map_err(anyhow::Error::msg)?;
        let inception = Self {
            version: ver,
            timestamp: Utc::now(),
            next_signers: next_signers,
            state: state,
        };
        inception.next_signers.validate()?;
        Ok(inception)
    }
}

mod tests {
    use cid::{multihash::Multihash, Cid};

    use crate::{signer::IdSigner, ED_CODE};

    use super::*;

    fn create_signer() -> Result<IdSigner> {
        let multihash = Multihash::<64>::wrap(0x12, b"test").unwrap(); // Create a Cid with codec ED_CODE
        let cid = Cid::new_v1(ED_CODE, multihash);
        // Value greater than 0
        let value = 1;
        // Attempt to create a new IdSigner
        let signer = IdSigner::new(value, cid.clone())?;
        Ok(signer)
    }

    #[test]
    fn incept_test() -> Result<()> {
        let signer = create_signer()?;
        let next_signers = IdNextSigners::new(1, vec![signer])?;
        let inception = IdInception::new("0.1.0", Cid::default(), next_signers)?;
        assert_eq!(inception.version, Version::parse("0.1.0").unwrap());
        Ok(())
    }
}
