use crate::{utils::sha256_hash, SHA2_256_CODE};
use cid::Cid;
use multihash::Multihash;
use regex::Regex;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone)]
pub struct SaidVersion {
    pub major: u16,
    pub minor: u16,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Said {
    pub version: SaidVersion,
    pub kind: String,
    pub cid: Cid,
}

#[derive(Debug)]
pub enum SaidError {
    InvalidIdFormat,
    PayloadAndIdNotMatch { expected: Vec<u8>, actual: Vec<u8> },
    InvalidHashAlg(u64),
    Unknown,
}

impl ToString for SaidError {
    fn to_string(&self) -> String {
        match self {
            SaidError::InvalidIdFormat => "invalid-id-format".to_string(),
            SaidError::PayloadAndIdNotMatch {
                expected: _,
                actual: _,
            } => format!("payload-and-id-not-match"),
            SaidError::InvalidHashAlg(alg) => format!("invalid-hash-alg: {}", alg),
            SaidError::Unknown => "unknown-error".to_string(),
        }
    }
}

impl Said {
    pub fn new(
        version: SaidVersion,
        kind: &str,
        codec: u64,
        bytes: &[u8],
    ) -> Result<Self, SaidError> {
        let input_digest = sha256_hash(bytes).map_err(|_| SaidError::Unknown)?;
        let mh =
            Multihash::<64>::wrap(SHA2_256_CODE, &input_digest).map_err(|_| SaidError::Unknown)?;
        let cid = Cid::new_v1(codec, mh);
        Ok(Self {
            version,
            kind: kind.to_string(),
            cid,
        })
    }

    pub fn validate(&self, payload: &[u8]) -> Result<&Self, SaidError> {
        match self.cid.hash().code() {
            SHA2_256_CODE => {
                let input_digest = sha256_hash(payload).map_err(|_| SaidError::Unknown)?;
                if self.cid.hash().digest() != input_digest.as_slice() {
                    return Err(SaidError::PayloadAndIdNotMatch {
                        expected: input_digest.to_vec(),
                        actual: self.cid.hash().digest().to_vec(),
                    });
                }
            }
            _ => return Err(SaidError::InvalidHashAlg(self.cid.hash().code())),
        }
        Ok(self)
    }
}

impl ToString for Said {
    fn to_string(&self) -> String {
        format!(
            "/idp2p/{}/{}/{}/{}",
            self.kind, self.version.major, self.version.minor, self.cid
        )
    }
}

impl FromStr for Said {
    type Err = SaidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re =
            Regex::new(r"^/idp2p/(?<kind>[a-z]+)/(?<major>\d+)/(?<minor>\d+)/(?<cid>.+)$").unwrap();

        let caps = re.captures(s).ok_or(Self::Err::InvalidIdFormat)?;

        let kind = caps["kind"].to_string();

        let major = caps["major"].to_string();
        let minor = caps["minor"].to_string();
        let version = SaidVersion {
            major: u16::from_str(major.as_str()).map_err(|_| Self::Err::InvalidIdFormat)?,
            minor: u16::from_str(minor.as_str()).map_err(|_| Self::Err::InvalidIdFormat)?,
        };
        let cid = caps["cid"].to_string();
        let cid = Cid::try_from(cid).map_err(|_| Self::Err::InvalidIdFormat)?;

        Ok(Self { version, kind, cid })
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
        let parsed = Said::from_str(input.as_str());
        assert!(parsed.is_ok());
        let idp2p_id = parsed.unwrap();
        // Check that version is Some("1.42")
        assert_eq!(
            idp2p_id.version,
            SaidVersion {
                major: 1,
                minor: 42
            }
        );
        // Check that kind is Event
        matches!(idp2p_id.kind.as_str(), "event");
    }

    #[test]
    fn test_invalid() {
        // Here, we include only a major version: /1/ instead of /1/2/
        let input = format!("/idp2p/event/{CID}");
        let parsed = Said::from_str(input.as_str());
        // Should fail because minor is missing
        assert!(parsed.is_err());
    }

    #[test]
    fn test_invalid_cid() {
        // Non-CID string as last segment
        let input = "/idp2p/id/1/2/not-a-cid";
        let parsed = Said::from_str(input);
        // Should fail because the CID parsing fails
        assert!(parsed.is_err());
    }

    #[test]
    fn test_invalid_payload() {
        // Non-CID string as last segment
        let input = "/idp2p/id/1/2/not-a-cid";
        let parsed = Said::from_str(input);
        // Should fail because the CID parsing fails
        assert!(parsed.is_err());
    }
}
