use idp2p_common::{
    anyhow::{bail, Result},
    cid::Cid,
    Idp2pCodec, Idp2pHasher,
};

use crate::IdentityError;

pub fn ensure_cid(cid: &[u8], inception: &[u8]) -> Result<()> {
    let cid: Cid = cid.to_vec().try_into()?;
    if cid.codec() != Idp2pCodec::Protobuf as u64 {
        bail!(IdentityError::InvalidId)
    }
    if !cid.hash().is_hash_of(inception)? {
        bail!(IdentityError::InvalidId)
    }
    Ok(())
}
