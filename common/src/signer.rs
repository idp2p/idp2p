use alloc::vec::Vec;
use anyhow::{bail, Result};
use cid::{multihash::Multihash, Cid};
use idp2p_utils::verifying::ED_CODE;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdSigner {
    pub value: u8,
    pub id: Cid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdNextSigners {
    pub quorum: u16,
    pub signers: Vec<IdSigner>,
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
    pub fn new(quorum: u16, signers: Vec<IdSigner>) -> Result<Self> {
        let next_signers = Self { quorum, signers };
        next_signers.validate()?;
        Ok(next_signers)
    }

    pub fn get_total_values(&self) -> u16 {
        self.signers.iter().map(|x| x.value as u16).sum()
    }

    pub fn validate(&self) -> Result<()> {
        if self.quorum == 0 {
            bail!("The quorum must be greater than 0.");
        }

        if self.get_total_values() < self.quorum {
            bail!("The quorum must be less than or equal to the total values of signers.");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use cid::Cid;
    use idp2p_utils::verifying::ED_CODE;

    #[test]
    fn test_id_signer_new_success() -> Result<()> {
        // Create a dummy multihash
        let multihash = Multihash::<64>::wrap(0x12, b"test").unwrap();        // Create a Cid with codec ED_CODE
        let cid = Cid::new_v1(ED_CODE, multihash);
        // Value greater than 0
        let value = 1;
        // Attempt to create a new IdSigner
        let signer = IdSigner::new(value, cid.clone())?;
        // Verify the fields
        assert_eq!(signer.value, value);
        assert_eq!(signer.id, cid);
        Ok(())
    }

    #[test]
    fn test_id_signer_new_zero_value() {
        let multihash = Multihash::<64>::wrap(0x12, b"my digest").unwrap();        // Create a Cid with codec ED_CODE
        let cid = Cid::new_v1(ED_CODE, multihash);
        // Value is 0, should fail
        let value = 0;
        let result = IdSigner::new(value, cid);
        assert!(result.is_err());
    }

    #[test]
    fn test_id_signer_new_invalid_codec() {
        let multihash = Multihash::<64>::wrap(0x12, b"my digest").unwrap();        // Create a Cid with codec ED_CODE
        // Use an invalid codec (e.g., 0x71 for dag-cbor)
        let invalid_codec = 0x71;
        let cid = Cid::new_v1(invalid_codec, multihash);
        let value = 1;
        let result = IdSigner::new(value, cid);
        assert!(result.is_err());
    }

    #[test]
    fn test_id_next_signers_new_success() -> Result<()> {
        let multihash = Multihash::<64>::wrap(0x12, b"my digest").unwrap();        // Create a Cid with codec ED_CODE
        let cid = Cid::new_v1(ED_CODE, multihash);
        let signer = IdSigner::new(1, cid)?;
        let signers = vec![signer];
        let quorum = 1;
        let next_signers = IdNextSigners::new(quorum, signers)?;
        assert_eq!(next_signers.quorum, quorum);
        Ok(())
    }

    #[test]
    fn test_id_next_signers_new_zero_quorum() {
        let multihash = Multihash::<64>::wrap(0x12, b"my digest").unwrap();        // Create a Cid with codec ED_CODE
        let cid = Cid::new_v1(ED_CODE, multihash);
        let signer = IdSigner::new(1, cid).unwrap();
        let signers = vec![signer];
        // Quorum is 0, should fail
        let quorum = 0;
        let result = IdNextSigners::new(quorum, signers);
        assert!(result.is_err());
    }

    #[test]
    fn test_id_next_signers_new_quorum_exceeds_total() {
        let multihash = Multihash::<64>::wrap(0x12, b"my digest").unwrap();        // Create a Cid with codec ED_CODE
        let cid = Cid::new_v1(ED_CODE, multihash);
        let signer = IdSigner::new(1, cid).unwrap();
        let signers = vec![signer];
        // Quorum exceeds total values of signers, should fail
        let quorum = 2;
        let result = IdNextSigners::new(quorum, signers);
        assert!(result.is_err());
    }

    #[test]
    fn test_id_next_signers_get_total_values() -> Result<()> {
        let multihash1 = Multihash::<64>::wrap(0x12, b"signer1").unwrap();        // Create a Cid with codec ED_CODE
        let cid1 = Cid::new_v1(ED_CODE, multihash1);
        let signer1 = IdSigner::new(2, cid1)?;

        let multihash2 = Multihash::<64>::wrap(0x12, b"signer2").unwrap();        // Create a Cid with codec ED_CODE
        let cid2 = Cid::new_v1(ED_CODE, multihash2);
        let signer2 = IdSigner::new(3, cid2)?;

        let signers = vec![signer1, signer2];
        let next_signers = IdNextSigners::new(4, signers)?;
        // Total values should be 5
        assert_eq!(next_signers.get_total_values(), 5);
        Ok(())
    }
}