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
    pub major: u16,
    pub minor: u16,
    pub cid: Cid,
}

#[derive(Error, Debug)]
pub enum IdError {
    #[error("Error")]
    InvalidIdFormat,
    #[error("Error")]
    PayloadAndIdNotMatch,
    #[error("Error")]
    InvalidHashAlg(u64),
    #[error("Error")]
    CommonError(#[from] CommonError),
    #[error("Error")]
    MultihashError(#[from] multihash::Error),
    #[error("Error")]
    Other(#[from] core::fmt::Error),
}

impl Id {
    pub fn new(
        kind: &str,
        major: u16,
        minor: u16,
        codec: u64,
        bytes: &[u8],
    ) -> Result<Self, IdError> {
        let input_digest = sha256_hash(bytes);
        let mh = Multihash::<64>::wrap(SHA2_256_CODE, &input_digest)?;
        let cid = Cid::new_v1(codec, mh);
        let kind = kind.to_string();
        Ok(Self {
            major,
            minor,
            kind,
            cid,
        })
    }

    pub fn version_str(&self) -> String {
        format!("{}.{}", self.major, self.minor)
    }

    pub fn validate(&self, payload: &[u8]) -> Result<&Self, IdError> {
        match self.cid.hash().code() {
            SHA2_256_CODE => {
                let input_digest = sha256_hash(payload);
                if self.cid.hash().digest() != input_digest.as_slice() {
                    return Err(IdError::PayloadAndIdNotMatch);
                }
            }
            _ => return Err(IdError::InvalidHashAlg(self.cid.hash().code())),
        }
        Ok(self)
    }
}

impl ToString for Id {
    fn to_string(&self) -> String {
        format!(
            "/idp2p/{}/{}/{}/{}",
            self.kind, self.major, self.minor, self.cid
        )
    }
}

impl FromStr for Id {
    type Err = IdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re =
            Regex::new(r"^/idp2p/(?<kind>[a-z]+)/(?<major>\d+)/(?<minor>\d+)/(?<cid>.+)$").unwrap();

        let caps = re.captures(s).ok_or(Self::Err::InvalidIdFormat)?;

        let kind = caps["kind"].to_string();

        let major = caps["major"].to_string();
        let minor = caps["minor"].to_string();
        let major = u16::from_str(major.as_str()).map_err(|_| Self::Err::InvalidIdFormat)?;
        let minor = u16::from_str(minor.as_str()).map_err(|_| Self::Err::InvalidIdFormat)?;
        let cid = caps["cid"].to_string();
        let cid = Cid::try_from(cid).map_err(|_| Self::Err::InvalidIdFormat)?;

        Ok(Self {
            major,
            minor,
            kind,
            cid,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // const ID: &str = "/did/1/idp2p/1/5464/event/bafkreieq5jui4j25lacwomsqgjeswwl3y5zcdrresptwgmfylxo2depppq";
    const CID: &str = "bafkreieq5jui4j25lacwomsqgjeswwl3y5zcdrresptwgmfylxo2depppq";

    #[test]
    fn test_valid_version() {
        let input = format!("/idp2p/event/1/42/{CID}");
        let parsed = Id::from_str(input.as_str());
        assert!(parsed.is_ok());
        let idp2p_id = parsed.unwrap();
        // Check that version is Some("1.42")
        assert_eq!(idp2p_id.major, 1);
        // Check that kind is Event
        matches!(idp2p_id.kind.as_str(), "event");
    }

    #[test]
    fn test_invalid() {
        // Here, we include only a major version: /1/ instead of /1/2/
        let input = format!("/idp2p/event/{CID}");
        let parsed = Id::from_str(input.as_str());
        // Should fail because minor is missing
        assert!(parsed.is_err());
    }

    #[test]
    fn test_invalid_cid() {
        // Non-CID string as last segment
        let input = "/idp2p/id/1/2/not-a-cid";
        let parsed = Id::from_str(input);
        // Should fail because the CID parsing fails
        assert!(parsed.is_err());
    }

    #[test]
    fn test_invalid_payload() {
        // Non-CID string as last segment
        let input = "/idp2p/id/1/2/not-a-cid";
        let parsed = Id::from_str(input);
        // Should fail because the CID parsing fails
        assert!(parsed.is_err());
    }
}
