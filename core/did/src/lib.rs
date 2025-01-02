use core::str::FromStr;

use error::DidError;
pub mod error;

/// DID
/// 
/// version is 0 for now and hidden
/// method is did method
/// kind, major, minor means the codec
/// 

#[derive(Debug, PartialEq, Clone)]
pub struct Did {
    pub kind: u16,
    pub major: u16,
    pub minor: u16,
    pub cid: Cid,
}

impl Did {
    pub fn to_bytes(&self) -> Vec<u8> {
        todo!()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DidError> {
        todo!()
    }
}


impl FromStr for Did {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id =  s.strip_prefix("/did/").ok_or(())?;
        let (_, did_bytes) = multibase::decode(id).map_err(|_|())?;
        let did = Self::from_bytes(&did_bytes)?;
        Ok(did)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
   
}
