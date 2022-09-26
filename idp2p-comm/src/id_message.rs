use idp2p_common::multi::{
    agreement::Idp2pAgreementKeypair, authentication::Idp2pAuthenticationKeypair,
};

use super::{error::Idp2pError, id_state::IdentityState};
pub type DecryptResult = Result<DecodeMessageResult, Idp2pError>;
#[derive(Debug, Clone, PartialEq)]
pub struct IdMessage {
    pub id: Vec<u8>,
    pub from: Vec<u8>,
    pub to: Vec<u8>,
    pub signer_kid: Vec<u8>,
    pub proof: Vec<u8>,
    pub created_at: i64,
    pub reply_to: Option<Vec<u8>>,
    pub body: IdMessageBody,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IdMessageBody {
    Text(String),
    Connect
}

#[derive(Debug, Clone, PartialEq)]
pub struct SealMessageInput {
    pub to: Vec<u8>,
    pub to_auth_keypair: Idp2pAuthenticationKeypair,
    pub from: Vec<u8>,
    pub from_agree_key: Idp2pAgreementKeypair,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SealMessageResult {
    pub message: Vec<u8>,
    pub shared_secret: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DecryptMessageResult {
    pub message: IdMessage,
    pub agreement_data: Vec<u8>,
}

pub trait MessageHandler {
    fn seal(&self, input: SealMessageInput) -> Result<SealMessageResult, Idp2pError>;
    fn decrypt(&self, kp: &Idp2pAuthenticationKeypair, msg: &[u8]) -> DecryptResult;
}
