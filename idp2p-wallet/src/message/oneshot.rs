#[derive(Debug, Clone, PartialEq)]
pub struct Idp2pOneShotMsgEnvelope {
    agreement_kid: Vec<u8>,
    agreement_hint : Vec<u8>,
    encryption_method: Vec<u8>,
    encryption_iv: Vec<u8>, 
    cipherbody : Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Idp2pOneShotMessage {
    pub from: Vec<u8>,
    pub to: Vec<u8>,
    pub signer_kid: Vec<u8>,
    pub proof: Vec<u8>,
    pub created_at: i64,
    pub body: OneShotMessageKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OneShotMessageKind {
    
}

#[derive(Debug, Clone, PartialEq)]
pub struct SealMessageResult {
    pub message: Vec<u8>,
    pub shared_secret: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DecryptMessageResult {
    pub message: Idp2pOneShotMessage,
    pub agreement_hint: Vec<u8>,
}

pub trait OneshotMessageHandler {
    fn seal(&self, to: &[u8], body: OneShotMessageKind) -> Result<SealMessageResult, ()>;
    fn decrypt(&self, msg: &[u8]) -> Result<DecryptMessageResult, ()>;
}