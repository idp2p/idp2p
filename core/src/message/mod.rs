use crate::{
    identity::Identity,
    multi::{base::Idp2pBase, hash::Idp2pHash, keypair::Idp2pKeypair},
};
pub mod codec;
pub struct IdMessage {
    id: Vec<u8>,
    body: Vec<u8>
}

impl IdMessage {
    pub fn new(to: &[u8], body: &[u8]) -> Self {

    }
    pub fn from_cipher() -> Self{

    }
    pub fn envelope(&self, signer: Idp2pKeypair, encrypter: Idp2pKeypair) -> Result<(), String> {
        Ok(())
    }
}
