use crate::signer::IdNextSigners;
use anyhow::Result;
use chrono::{DateTime, Utc};
use cid::Cid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdEvent {
    pub version: String,
    pub timestamp: DateTime<Utc>,
    pub previous: Cid,
    pub signers: Vec<Cid>,
    pub next_signers: IdNextSigners,
    pub state: Cid,
}

impl IdEvent {
    pub fn new(
        version: &str,
        state: Cid,
        previous: Cid,
        signers: Vec<Cid>,
        next_signers: IdNextSigners,
    ) -> Result<Self> {
        let ver = semver::Version::parse(version).map_err(anyhow::Error::msg)?;
        let event = Self {
            version: ver.to_string(),
            timestamp: Utc::now(),
            previous: previous,
            signers: signers,
            next_signers: next_signers,
            state: state,
        };
        event.next_signers.validate()?;

        Ok(event)
    }
}

mod tests {
    use cid::{multihash::Multihash, Cid};
    use idp2p_utils::verifying::ED_CODE;

    use crate::signer::IdSigner;

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
    fn event_test() -> Result<()> {
        let signer = create_signer()?;
        let previous = Cid::new_v1(ED_CODE, Multihash::<64>::wrap(0x12, b"test").unwrap());
        let next_signers = IdNextSigners::new(1, vec![signer.clone()])?;
        let event = IdEvent::new(
            "0.1.0",
            Cid::default(),
            previous,
            vec![signer.id],
            next_signers,
        )?;
        assert_eq!(event.version, "0.1.0");
        Ok(())
    }
}
