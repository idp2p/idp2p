use crate::store::IdSecret;

use super::error::IdentityError;

#[derive(Debug, Clone, PartialEq)]
pub struct IdMessage {
    from: Vec<u8>,
    signer_kid: Vec<u8>,
    proof: Vec<u8>,
    to: Vec<u8>,
    created_at: i64,
    body: Vec<u8>,
}

pub trait MessageHandler {
    fn seal_msg(&self, msg: IdMessage, signer_secret: IdSecret) -> Result<Vec<u8>, IdentityError>;
    fn decode_msg(&self, msg: &[u8], agree_secret: IdSecret) -> Result<IdMessage, IdentityError>;
}
