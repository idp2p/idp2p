use cid::Cid;
use multihash::Multihash;


// [kind]([major][minor])?
impl Idp2pIdKind {
    fn encode(&self) -> Vec<u8> {
        match self {
            Idp2pIdKind::Id(version) => 0,
            Idp2pIdKind::Event(version) => 1,
            Idp2pIdKind::Signer => 2,
            Idp2pIdKind::Message(version) => 3,
            Idp2pIdKind::Mediator => 4,
            Idp2pIdKind::Peer => 5,
            Idp2pIdKind::Authentication => 6,
            Idp2pIdKind::KeyAgreement => 7,
            Idp2pIdKind::AssertionMethod => 8,
        
        }
    }
}

impl Idp2pId {
    pub fn new(kind: Idp2pIdKind, codec: u64, bytes: &[u8]) -> Result<Self, IdError> {
        let prefix = kind.encode();
        let input_digest = sha256_hash(bytes)?;
        let mh =
            Multihash::<64>::wrap(SHA2_256_CODE, &input_digest).map_err(|_| IdError::Unknown)?;
        let cid = Cid::new_v1(codec, mh);
        todo!()
    }
}


pub trait CidExt {
    fn ensure(&self, input: &[u8]) -> Result<(), IdError>;
    fn create(code: u64, input: &[u8]) -> Result<Cid, IdError>;
}

impl CidExt for Cid {
    fn ensure(&self, input: &[u8]) -> Result<(), IdError> {
        match self.hash().code() {
            SHA2_256_CODE => {
                let input_digest = sha256_hash(input)?;
                if self.hash().digest() != input_digest.as_slice() {
                    return Err(IdError::EnsureError {
                        expected: input_digest.to_vec(),
                        actual: self.hash().digest().to_vec(),
                    });
                }
            }
            _ => return Err(IdError::InvalidHashAlg(self.hash().code())),
        }
        Ok(())
    }

    fn create(code: u64, input: &[u8]) -> Result<Self, IdError> {
        let input_digest = sha256_hash(input)?;
        let mh =
            Multihash::<64>::wrap(SHA2_256_CODE, &input_digest).map_err(|_| IdError::Unknown)?;
        Ok(Cid::new_v1(code, mh))
    }
}


use std::str::FromStr;
use regex::Regex;
use crate::error::IdError;

pub struct Idp2pId {
    pub kind: String,
    pub identifier: String,
    pub version: Option<String>,
}

impl FromStr for Idp2pId {
    type Err = IdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // If major is present, minor must also be present and vice versa
        //
        let re =
            Regex::new(r"^/idp2p/(?<kind>[a-z]+)(?:/(?<major>\d+)/(?<minor>\d+))?/(?<identifier>.+)$")
                .unwrap();

        let caps = re.captures(s).ok_or(IdError::InvalidId)?;

        let kind = caps["kind"].to_string();

        let major = caps.name("major").map(|m| m.as_str());
        let minor = caps.name("minor").map(|m| m.as_str());

        let version = match (major, minor) {
            (Some(maj), Some(min)) => Some(format!("{}.{}", maj, min)),
            (None, None) => None,
            _ => return Err(IdError::InvalidId),
        };

        let identifier = caps["identifier"].to_string();
        //let cid = Cid::try_from(cid_str).map_err(|_| IdError::InvalidId)?;

        Ok(Self { identifier, kind, version })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    const CID: &str = "bafkreieq5jui4j25lacwomsqgjeswwl3y5zcdrresptwgmfylxo2depppq";
    // Helper function to quickly test parsing
    fn parse_idp2p_id(input: &str) -> Result<Idp2pId, IdError> {
        Idp2pId::from_str(input)
    }

    #[test]
    fn test_valid_no_version() {
        let input = format!("/idp2p/id/{CID}");
        let parsed = parse_idp2p_id(input.as_str());
        assert!(parsed.is_ok());
        let idp2p_id = parsed.unwrap();
        // Check that version is None
        assert_eq!(idp2p_id.version, None);
        // Check that kind is Id
        matches!(idp2p_id.kind.as_str(), "id");
    }

    #[test]
    fn test_valid_version() {
        let input = format!("/idp2p/event/1/42/{CID}");
        let parsed = parse_idp2p_id(input.as_str());
        assert!(parsed.is_ok());
        let idp2p_id = parsed.unwrap();
        // Check that version is Some("1.42")
        assert_eq!(idp2p_id.version, Some("1.42".to_string()));
        // Check that kind is Event
        matches!(idp2p_id.kind.as_str(), "event");
    }

    #[test]
    fn test_invalid_only_major() {
        // Here, we include only a major version: /1/ instead of /1/2/
        let input = format!("/idp2p/event/1/{CID}");
        let parsed = parse_idp2p_id(input.as_str());
        // Should fail because minor is missing
        assert!(parsed.is_err());
    }

    #[test]
    fn test_invalid_only_minor() {
        // This doesn't even match the required pattern, but let's see it fail
        let input = format!("/idp2p/event/1.2/{CID}");
        let parsed = parse_idp2p_id(input.as_str());
        // Should fail because major is missing
        assert!(parsed.is_err());
    }

    #[test]
    fn test_invalid_cid() {
        // Non-CID string as last segment
        let input = "/idp2p/id/1/2/not-a-cid";
        let parsed = parse_idp2p_id(input);
        // Should fail because the CID parsing fails
        assert!(parsed.is_err());
    }
}
