use crate::jwk::Jwk;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Jwe {
    pub iv: String, // random initial vector 12 bytes
    pub ciphertext: String, // Encrypted message 
    pub receipents: Vec<>
}

struct JweProtected{
    pub typ: String, // application/didcomm-encrypted+json
    pub enc: String, // XC20P
    pub alg: String, // ECDH-ES+A256KW
    pub epk: Jwk,  // sender public key
}

impl Jwe{
    pub fn new(kid: &str, jwm: Jwm) -> Jwe{
        // create a secret and epk 
        // create iv
        //  
    }
}