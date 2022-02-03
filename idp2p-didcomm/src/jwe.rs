use crate::jwk::Jwk;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Jwe {
    pub typ: String, // application/didcomm-encrypted+json
    pub enc: String, // XC20P
    pub alg: String, // ECDH-ES+A256KW
    pub kid: String, // receipent kid
    pub epk: Jwk,  // sender public key
    pub iv: String, // initial vector 12 bytes
    pub ciphertext: String, // Encrypted message 
}

impl Jwe{
    pub fn new(kid: &str, jwm: Jwm) -> Jwe{
        // create a secret and epk 
        // create iv
        //  
    }
}