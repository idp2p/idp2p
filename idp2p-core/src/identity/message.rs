use idp2p_common::multi::{agreement_secret::Idp2pAgreementSecret, key_secret::Idp2pKeySecret};

use super::{error::IdentityError, state::IdentityState};

#[derive(Debug, Clone, PartialEq)]
pub struct IdMessage {
    from: Vec<u8>,
    to: Vec<u8>,
    signer_kid: Vec<u8>,
    proof: Vec<u8>,
    created_at: i64,
    body: Vec<u8>,
}

pub trait MessageHandler {
    fn seal_msg(
        &self,
        auth_secret: Idp2pKeySecret,
        from: IdentityState,
        to: IdentityState,
        body: &[u8],
    ) -> Result<Vec<u8>, IdentityError>;
    fn decode_msg(
        &self,
        agree_secret: Idp2pAgreementSecret,
        msg: &[u8],
    ) -> Result<IdMessage, IdentityError>;
}
