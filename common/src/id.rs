use crate::{error::CommonError, utils::sha256_hash, SHA2_256_CODE};
use alloc::string::{String, ToString};
use cid::Cid;
use core::str::FromStr;
use multihash::Multihash;
use regex::Regex;
use thiserror::Error;

#[derive(Debug, PartialEq, Clone)]
pub struct Id {
    pub kind: String,
    pub cid: Cid,
}

#[derive(Error, Debug)]
pub enum IdError {
    #[error("Invalid ID format. Expected format: /idp2p/<kind>/<cid>")]
    InvalidIdFormat,
    
    #[error("Payload hash does not match the CID hash")]
    PayloadHashMismatch,
    
    #[error("Unsupported hash algorithm: {0}. Expected SHA2-256")]
    UnsupportedHashAlgorithm(u64),
    
    #[error("Invalid kind: {0}. Kind must be a lowercase alphabetic string")]
    InvalidKind(String),
    
    #[error("CID parsing error: {0}")]
    InvalidCid(String),
    
    #[error("Common error: {0}")]
    Common(#[from] CommonError),
    
    #[error("Multihash error: {0}")]
    Multihash(#[from] multihash::Error),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl Id {
    pub fn new(
        kind: &str,
        codec: u64,
        bytes: &[u8],
    ) -> Result<Self, IdError> {
        if !kind.chars().all(|c| c.is_ascii_lowercase()) {
            return Err(IdError::InvalidKind(kind.to_string()));
        }

        let input_digest = sha256_hash(bytes);
        let mh = Multihash::<64>::wrap(SHA2_256_CODE, &input_digest)
            .map_err(|e| IdError::Multihash(e))?;
        let cid = Cid::new_v1(codec, mh);
        let kind = kind.to_string();
        
        Ok(Self { kind, cid })
    }

    pub fn ensure(&self, payload: &[u8]) -> Result<&Self, IdError> {
        let hash_code = self.cid.hash().code();
        if hash_code != SHA2_256_CODE {
            return Err(IdError::UnsupportedHashAlgorithm(hash_code));
        }

        let input_digest = sha256_hash(payload);
        if self.cid.hash().digest() != input_digest.as_slice() {
            return Err(IdError::PayloadHashMismatch);
        }

        Ok(self)
    }
}

impl ToString for Id {
    fn to_string(&self) -> String {
        format!("/idp2p/{}/{}", self.kind, self.cid)
    }
}

impl FromStr for Id {
    type Err = IdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r"^/idp2p/(?<kind>[a-z]+)/(?<cid>.+)$")
            .map_err(|e| IdError::Internal(e.to_string()))?;

        let caps = re.captures(s)
            .ok_or(IdError::InvalidIdFormat)?;

        let kind = caps["kind"].to_string();
        if !kind.chars().all(|c| c.is_ascii_lowercase()) {
            return Err(IdError::InvalidKind(kind));
        }

        let cid_str = caps["cid"].to_string();
        let cid = Cid::try_from(cid_str.as_str())
            .map_err(|e| IdError::InvalidCid(e.to_string()))?;

        Ok(Self { kind, cid })
    }
}
// did:p2p:{cid}
#[cfg(test)]
mod tests {
    use super::*;
    const CID: &str = "bafkreieq5jui4j25lacwomsqgjeswwl3y5zcdrresptwgmfylxo2depppq";

    #[test]
    fn test_valid_id() {
        let input = format!("/idp2p/event/{CID}");
        let parsed = Id::from_str(&input);
        assert!(parsed.is_ok());
        let id = parsed.unwrap();
        assert_eq!(id.kind, "event");
        assert_eq!(id.to_string(), input);
    }

    #[test]
    fn test_invalid_prefix() {
        let input = format!("/abc/event/{CID}");
        let error = Id::from_str(&input).unwrap_err();
        assert!(matches!(error, IdError::InvalidIdFormat));
    }

    #[test]
    fn test_invalid_kind() {
        let input = format!("/idp2p/EVENT/{CID}");
        let error = Id::from_str(&input).unwrap_err();
        assert!(matches!(error, IdError::InvalidIdFormat));
    }

    #[test]
    fn test_invalid_cid() {
        let input = "/idp2p/event/not-a-cid";
        let error = Id::from_str(input).unwrap_err();
        assert!(matches!(error, IdError::InvalidCid(_)));
    }

    #[test]
    fn test_new_invalid_kind() {
        let result = Id::new("INVALID", 0, &[0u8; 32]);
        assert!(matches!(result, Err(IdError::InvalidKind(_))));
    }

    #[test]
    fn test_validate_wrong_hash() {
        let id = Id::new("test", 0, &[0u8; 32]).unwrap();
        let result = id.ensure(&[1u8; 32]);
        assert!(matches!(result, Err(IdError::PayloadHashMismatch)));
    }
}