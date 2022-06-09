use idp2p_common::{
    chrono::Utc,
    multi::{
        id::{Idp2pCid, Idp2pCodec},
        keypair::Idp2pKeypair,
    }, random::create_random,
};

use self::error::IdMessageError;
pub mod error;
pub mod protobuf;
pub mod reqres;

pub trait IdMessageBehaviour {
    fn encode(&self, codec: Idp2pCodec) -> Result<Vec<u8>, IdMessageError>;
    fn decode(&self, msg: &[u8]) -> Result<IdMessage, IdMessageError>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdMessage {
    id: Vec<u8>,
    from: Vec<u8>,
    to: Vec<u8>,
    created_at: i64,
    body: Vec<u8>,
}

impl IdMessage {
    /// Create a raw message.
    ///
    /// * `input` - Message properties.
    pub fn new(from: &[u8], to: &[u8], body: &[u8]) -> Self {
        let id: [u8; 32] = create_random();
        IdMessage {
            id: id.to_vec(),
            from: from.to_vec(),
            to: to.to_vec(),
            created_at: Utc::now().timestamp(),
            body: body.to_vec(),
        }
    }

    /// Receive a message.
    ///
    /// Decoding, decrypting and verifying encoded message,
    ///   
    /// * `msg` - Encoded message.
    /// * `kp` - Agreement key pair to create shared secret. It should be conveinent with agreement_kid
    pub fn receive(&self, msg: &[u8], kp: Idp2pKeypair) -> Result<Self, IdMessageError> {
        // get codec
        todo!()
    }

    /// Seal message.
    ///
    /// Signing, encrypting, and encoding message,
    ///  
    /// * `codec` - ie protobuf, json.
    /// * `kp` - Signing key pair to sign message
    pub fn seal(&self, codec: Idp2pCodec, kp: Idp2pKeypair) -> Vec<u8> {
        todo!()
    }
}
