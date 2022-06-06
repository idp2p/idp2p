use super::error::Idp2pMultiError;

pub enum Idp2pBase {
    Base58Btc,
}

impl Default for Idp2pBase {
    fn default() -> Self {
        Idp2pBase::Base58Btc
    }
}

impl Idp2pBase {
    pub fn encode<T: AsRef<[u8]>>(&self, bytes: T) -> String {
        match self {
            Idp2pBase::Base58Btc => multibase::encode(multibase::Base::Base58Btc, bytes),
        }
    }

    pub fn decode(s: &str) -> Result<Vec<u8>, Idp2pMultiError>{
        Ok(multibase::decode(s)?.1)
    }
}
