use idp2p_common::multi::{agreement_secret::Idp2pAgreementSecret, key_secret::Idp2pKeySecret};

use super::{error::Idp2pError, id_state::IdentityState};

#[derive(Debug, Clone, PartialEq)]
pub struct IdMessage {
    pub(crate) from: Vec<u8>,
    pub(crate) to: Vec<u8>,
    pub(crate) signer_kid: Vec<u8>,
    pub(crate) proof: Vec<u8>,
    pub(crate) created_at: i64,
    pub(crate) body: Vec<u8>,
}

pub trait MessageHandler {
    fn seal_msg(
        &self,
        auth_secret: Idp2pKeySecret,
        from: IdentityState,
        to: IdentityState,
        body: &[u8],
    ) -> Result<Vec<u8>, Idp2pError>;
    fn decode_msg(
        &self,
        agree_secret: Idp2pAgreementSecret,
        msg: &[u8],
    ) -> Result<IdMessage, Idp2pError>;
}