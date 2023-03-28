use super::error::Idp2pMultiError;

pub enum Idp2pBase {
    Base58Btc,
    Hex
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
            Idp2pBase::Hex => multibase::encode(multibase::Base::Base16Lower, bytes),
        }
    }

    pub fn decode(s: &str) -> Result<Vec<u8>, Idp2pMultiError> {
        Ok(multibase::decode(s)?.1)
    }

    pub fn decode_sized<const N: usize>(s: &str) -> Result<[u8; N], Idp2pMultiError> {
        let r = multibase::decode(s)?.1;
        let data: [u8; N] = r.try_into().expect("Data size is not equal to given size");
        Ok(data)
    }
}
