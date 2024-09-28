use alloc::vec::{self, Vec};
use anyhow::{bail, Result};
use cid::{multihash::Multihash, Cid};
use idp2p_utils::verifying::ED_CODE;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct IdSigner {
    pub value: u8,
    pub id: Cid
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdNextSigners {
    pub quorum: u8,
    pub signers: Vec<IdSigner>
}

impl IdSigner {

    /// Creates a new `IdSigner` instance after validating.
    pub fn new(value: u8, id: Cid) -> Result<Self> {
        let signer = Self { value, id };
        signer.validate()?;
        Ok(signer)
    }

    pub fn validate(&self) -> Result<()> {
        if self.value == 0 {
            bail!("The order of the signer must be greater than 0.");
        }
        if self.id.codec() != ED_CODE {
            bail!("The codec of the signer must be ed.");
        }
        Ok(())
    }
}

impl IdNextSigners {
    pub fn new(quorum: u8, signers: Vec<IdSigner>) -> Result<Self> {        
        let next_signers = Self { quorum, signers };
        next_signers.validate()?;
        Ok(next_signers)
    }

    pub fn get_total_values(&self) -> u8 {
        self.signers.iter().map(|x| x.value).sum()
    }

    pub fn validate(&self) -> Result<()> {
        if self.quorum == 0 {   
            bail!("The quorum must be greater than 0.");
        }        
        if self.get_total_values() <= self.quorum {
            bail!("The quorum must be less than or equal to the total values of signers.");
        }
        
        Ok(())
    }
}

#[test]
fn test_id_signer() {
    let id = Cid::new_v1(ED_CODE, Multihash::default());
    let signer = IdSigner::new(1, id).unwrap();
    let next_signers = IdNextSigners::new(4, vec![signer]).unwrap();

    assert!(next_signers.validate().is_ok());
}