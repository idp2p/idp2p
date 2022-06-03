use crate::{identity::Identity, multi::{hash::Idp2pHash, base::Idp2pBase, keypair::Idp2pKeypair}};

pub struct Idp2pMessage{
   id: Vec<u8>,
   from: Vec<u8>,
   to: Vec<u8>,
   body: Vec<u8>,
   created_at: i64
}

pub struct Idp2pConfig {
   id: Vec<u8>,
   hash: Idp2pHash,
   base: Idp2pBase,
   auth_keypair: Idp2pKeypair,
   agree_keypair: Idp2pKeypair 
}
 
pub mod codec;
pub mod store;