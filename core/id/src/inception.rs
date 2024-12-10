use chrono::prelude::*;
use cid::Cid;
use idp2p_common::{cid::CidExt, ED_CODE};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::{IdConfig, IdMultisig};

/// IdInception
///
/// The inception of the identity protocol.
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdInception {
    pub config: IdConfig,
    pub state: Cid,
    pub timestamp: DateTime<Utc>,
    pub next_signers: Vec<Cid>,
    pub mediators: Vec<Cid>,
}

impl IdInception {
    pub fn generate(signer: &[u8], mediator: &str) -> anyhow::Result<Self> {
        let mediator = Cid::from_str(mediator).map_err(anyhow::Error::msg)?;
        let state = cid::Cid::default();
        let signer = Cid::create(ED_CODE, signer)?;
        let next_signers = vec![signer];

        let inception = IdInception {
            config: IdConfig {
               multisig:  IdMultisig::OneOfOne,
               recovery_duration: 60*60*24
            },
            state,
            timestamp: Utc::now(),
            next_signers,
            mediators: vec![mediator],
        };

        Ok(inception)
    }
}

mod tests {
    use super::*;

    #[test]
    fn incept_test() -> anyhow::Result<()> {
        let inception = IdInception::generate(b"signer", &Cid::default().to_string())?;
        assert_eq!(inception.next_signers.len(), 1);
        Ok(())
    }
}
