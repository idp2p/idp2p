#[derive(Debug)]
pub enum Idp2pSecret{
    Ed25519{
        secret: ed25519_dalek::SecretKey
    }
}

impl Idp2pSecret{
    pub fn sign(){

    }
    pub fn to_shared_secret(&self, public: [u8; 32]) -> Vec<u8>{
       todo!()
    }
}