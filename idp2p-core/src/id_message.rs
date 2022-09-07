use idp2p_common::multi::{ agreement::Idp2pAgreementKeypair, authentication::Idp2pAuthenticationKeypair};

use super::{error::Idp2pError, id_state::IdentityState};

#[derive(Debug, Clone, PartialEq)]
pub struct IdMessage {
    pub from: Vec<u8>,
    pub to: Vec<u8>,
    pub signer_kid: Vec<u8>,
    pub proof: Vec<u8>,
    pub created_at: i64,
    pub body: Vec<u8>,
    pub reply_to: Option<Vec<u8>>
}

pub trait MessageHandler {
    fn seal_msg(
        &self,
        auth_keypair: Idp2pAuthenticationKeypair,
        from: IdentityState,
        to: IdentityState,
        body: &[u8],
    ) -> Result<Vec<u8>, Idp2pError>;
    fn decode_msg(
        &self,
        agree_keypair: Idp2pAgreementKeypair,
        msg: &[u8],
    ) -> Result<(IdMessage, Vec<u8>), Idp2pError>;
}
