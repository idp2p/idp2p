use chrono::prelude::*;
use cid::Cid;
use idp2p_common::{cbor, cid::CidExt, ED_CODE};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::{IdError, IdErrorCode, IdMultisig};

/// IdInception
///
/// The inception of the identity protocol.
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdInception {
    pub multisig: IdMultisig,
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
            multisig: IdMultisig::OneOfOne,
            state,
            timestamp: Utc::now(),
            next_signers,
            mediators: vec![mediator],
        };

        Ok(inception)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, IdError> {
        let inception = cbor::decode(bytes)?;
        Ok(inception)
    }

    pub fn validate(&self) -> Result<(), IdError> {
        let total_signers: u16 = self.next_signers.len() as u16;
        if total_signers !=  self.multisig.total_signers() {
            return  Err(IdError{
                code: IdErrorCode::Other,
                message: format!("The number of signers must be {}.", self.multisig.total_signers()),
            });
        }
        for signer in &self.next_signers {
            if signer.codec() != ED_CODE {
                return  Err(IdError{
                    code: IdErrorCode::Other,
                    message: format!("The signer codec must be {}.", ED_CODE),
                });
            }
        }
            
        Ok(())
    }
}

mod tests {
    use super::*;

    #[test]
    fn incept_test() -> anyhow::Result<()> {
        let inception = IdInception::generate(b"signer", &Cid::default().to_string())?;
        inception.validate()?;
        assert_eq!(inception.next_signers.len(), 1);
        Ok(())
    }
}
