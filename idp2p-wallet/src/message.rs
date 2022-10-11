use self::oneshot::Idp2pOneShotMsgEnvelope;

mod oneshot;
pub enum Idp2pPubsubMessage{
    ResolveReq(String),
    ResolveRes(Vec<u8>),
    OneshotMsgEnvelope(Idp2pOneShotMsgEnvelope)
}