use idp2p_common::anyhow::*;
use idp2p_common::base64url;
use idp2p_common::serde_json;
use serde::{Deserialize, Serialize};

const KTY: &str = "OKP";
const CRV: &str = "X25519";
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Jwk {
    pub kty: String,
    pub crv: String,
    pub x: String,
}

impl Jwk {
    pub fn from_bytes(bytes: [u8; 32]) -> Result<Self> {
        let mb64 = base64url::encode_bytes(&bytes)?;
        Ok(Jwk {
            kty: KTY.to_owned(),
            crv: CRV.to_owned(),
            x: mb64[1..].to_owned(),
        })
    }

    pub fn from_str(s: &str) -> Result<Self> {
        let jwk: Jwk = serde_json::from_str(s)?;
        if jwk.kty != KTY {
            bail!("kty should be {}", KTY)
        }
        if jwk.crv != CRV {
            bail!("crv should be {}", CRV)
        }
        Ok(jwk)
    }

    pub fn to_vec(&self) -> [u8; 32] {
        let mb64 = format!("u{}", &self.x);
        idp2p_common::decode_sized::<32>(&mb64).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn from_test() {
        let jwk = Jwk::from_bytes([0; 32]).unwrap();
        assert_eq!(jwk.kty, "OKP");
        assert_eq!(jwk.crv, "X25519");
        assert_eq!(jwk.x, "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
        let v = jwk.to_vec();
        assert_eq!(v, [0; 32]);
    }
}
