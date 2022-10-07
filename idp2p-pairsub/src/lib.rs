use async_trait::async_trait;
use idp2p_proto::{
    pairsub_request::{PairsubRequestKind as ProtoPairsubRequestKind, SubscribeMessage},
    pairsub_response::{GetOk, PairsubResponseError, ResponseStatus},
    pubsub_message::PubsubMessage as ProtoPubsubMessage,
};
use libp2p::{
    core::upgrade::{read_length_prefixed, write_length_prefixed},
    futures::{AsyncRead, AsyncWrite, AsyncWriteExt},
    request_response::{ProtocolName, RequestResponseCodec},
};
use prost::Message;

pub mod idp2p_proto {
    include!(concat!(env!("OUT_DIR"), "/idp2p.pb.rs"));
}

#[derive(Debug, Clone, PartialEq)]
pub enum PairsubMessageKind {
    Resolve,
    Publish(Vec<u8>),
    Envelope(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PairsubMessage {
    topic: String,
    message: PairsubMessageKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PairsubRequestKind {
    Get,
    Subscribe(Vec<String>),
    Publish(PairsubMessage),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PairsubRequest {
    pub access_token: String,
    pub message: PairsubRequestKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PairsubResponse {
    Ok,                         // Subscribe or Publish
    GetOk(Vec<PairsubMessage>), // Get messages
    Error {
        id: String,
        code: String,
        message: String,
    },
}

#[derive(Debug, Clone)]
pub struct PairsubProtocol();
#[derive(Clone)]
pub struct PairsubCodec();

impl ProtocolName for PairsubProtocol {
    fn protocol_name(&self) -> &[u8] {
        "/idp2p/pairsub/1".as_bytes()
    }
}

impl Into<PairsubMessage> for idp2p_proto::PairsubMessage {
    fn into(self) -> PairsubMessage {
        let kind = match self.message.unwrap().pubsub_message.unwrap() {
            ProtoPubsubMessage::Resolve(_) => PairsubMessageKind::Resolve,
            ProtoPubsubMessage::Publish(payload) => PairsubMessageKind::Publish(payload),
            ProtoPubsubMessage::Envelope(payload) => PairsubMessageKind::Envelope(payload),
        };
        PairsubMessage {
            topic: self.topic,
            message: kind,
        }
    }
}

impl Into<idp2p_proto::PairsubMessage> for PairsubMessage {
    fn into(self) -> idp2p_proto::PairsubMessage {
        let pubsub_msg = match self.message {
            PairsubMessageKind::Resolve => ProtoPubsubMessage::Resolve(true),
            PairsubMessageKind::Publish(payload) => ProtoPubsubMessage::Publish(payload),
            PairsubMessageKind::Envelope(payload) => ProtoPubsubMessage::Envelope(payload),
        };
        idp2p_proto::PairsubMessage {
            topic: self.topic,
            message: Some(idp2p_proto::PubsubMessage {
                pubsub_message: Some(pubsub_msg),
            }),
        }
    }
}

impl Into<PairsubRequest> for idp2p_proto::PairsubRequest {
    fn into(self) -> PairsubRequest {
        let kind = match self.pairsub_request_kind.unwrap() {
            ProtoPairsubRequestKind::Get(_) => PairsubRequestKind::Get,
            ProtoPairsubRequestKind::Subscribe(msg) => PairsubRequestKind::Subscribe(msg.topics),
            ProtoPairsubRequestKind::Publish(msg) => PairsubRequestKind::Publish(msg.into()),
        };
        PairsubRequest {
            access_token: self.access_token,
            message: kind,
        }
    }
}

impl Into<idp2p_proto::PairsubRequest> for PairsubRequest {
    fn into(self) -> idp2p_proto::PairsubRequest {
        let kind = match self.message {
            PairsubRequestKind::Get => ProtoPairsubRequestKind::Get(true),
            PairsubRequestKind::Subscribe(topics) => {
                ProtoPairsubRequestKind::Subscribe(SubscribeMessage { topics: topics })
            }
            PairsubRequestKind::Publish(msg) => ProtoPairsubRequestKind::Publish(msg.into()),
        };
        idp2p_proto::PairsubRequest {
            access_token: self.access_token,
            pairsub_request_kind: Some(kind),
        }
    }
}

impl Into<PairsubResponse> for idp2p_proto::PairsubResponse {
    fn into(self) -> PairsubResponse {
        match self.response_status.unwrap() {
            ResponseStatus::Ok(_) => PairsubResponse::Ok,
            ResponseStatus::GetOk(res) => {
                let mut messages: Vec<PairsubMessage> = vec![];
                for msg in res.messages {
                    messages.push(msg.into());
                }
                PairsubResponse::GetOk(messages)
            }
            ResponseStatus::Error(err) => PairsubResponse::Error {
                id: err.id,
                code: err.code,
                message: err.message,
            },
        }
    }
}

impl Into<idp2p_proto::PairsubResponse> for PairsubResponse {
    fn into(self) -> idp2p_proto::PairsubResponse {
        let status = match self {
            PairsubResponse::Ok => ResponseStatus::Ok(true),
            PairsubResponse::GetOk(messages) => {
                let mut p_messages: Vec<idp2p_proto::PairsubMessage> = vec![];
                for msg in messages {
                    p_messages.push(msg.into());
                }
                ResponseStatus::GetOk(GetOk {
                    messages: p_messages,
                })
            }
            PairsubResponse::Error { id, code, message } => {
                ResponseStatus::Error(PairsubResponseError { id, code, message })
            }
        };
        idp2p_proto::PairsubResponse {
            response_status: Some(status),
        }
    }
}

#[async_trait]
impl RequestResponseCodec for PairsubCodec {
    type Protocol = PairsubProtocol;
    type Request = PairsubRequest;
    type Response = PairsubResponse;

    async fn read_request<T>(
        &mut self,
        _: &PairsubProtocol,
        io: &mut T,
    ) -> std::io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let vec = read_length_prefixed(io, 1_000_000).await?;

        if vec.is_empty() {
            return Err(std::io::ErrorKind::UnexpectedEof.into());
        }
        let req_proto = idp2p_proto::PairsubRequest::decode(&*vec)?;
        Ok(req_proto.into())
    }

    async fn read_response<T>(
        &mut self,
        _: &PairsubProtocol,
        io: &mut T,
    ) -> std::io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let vec = read_length_prefixed(io, 1_000_000).await?;

        if vec.is_empty() {
            return Err(std::io::ErrorKind::UnexpectedEof.into());
        }
        let res_proto = idp2p_proto::PairsubResponse::decode(&*vec)?;
        Ok(res_proto.into())
    }

    async fn write_request<T>(
        &mut self,
        _: &PairsubProtocol,
        io: &mut T,
        req: Self::Request,
    ) -> std::io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let req_proto: idp2p_proto::PairsubRequest = req.into();
        write_length_prefixed(io, req_proto.encode_to_vec()).await?;
        io.close().await?;
        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _: &PairsubProtocol,
        io: &mut T,
        res: Self::Response,
    ) -> std::io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let res_proto: idp2p_proto::PairsubResponse = res.into();
        write_length_prefixed(io, res_proto.encode_to_vec()).await?;
        io.close().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pairsub_message_map_test(){
        let msg = PairsubMessage{
            topic: "Some".to_string(),
            message: PairsubMessageKind::Publish(vec![0u8;32])
        };
        let proto_msg:idp2p_proto::PairsubMessage = msg.clone().into();
        let encoded = proto_msg.encode_to_vec();
        let decoded: PairsubMessage = idp2p_proto::PairsubMessage::decode(&*encoded).unwrap().into();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn pairsub_request_map_test(){
        let msg = PairsubMessage{
            topic: "Some".to_string(),
            message: PairsubMessageKind::Publish(vec![0u8;32])
        };
        let req = PairsubRequest{
            access_token: "Some".to_string(),
            message: PairsubRequestKind::Publish(msg)
        };
        let proto_req:idp2p_proto::PairsubRequest = req.clone().into();
        let encoded = proto_req.encode_to_vec();
        let decoded: PairsubRequest = idp2p_proto::PairsubRequest::decode(&*encoded).unwrap().into();
        assert_eq!(req, decoded);
    }

    #[test]
    fn pairsub_response_map_test(){
        let msg = PairsubMessage{
            topic: "Some".to_string(),
            message: PairsubMessageKind::Publish(vec![0u8;32])
        };
        let res = PairsubResponse::GetOk(vec![msg]);
        let proto_res:idp2p_proto::PairsubResponse = res.clone().into();
        let encoded = proto_res.encode_to_vec();
        let decoded: PairsubResponse = idp2p_proto::PairsubResponse::decode(&*encoded).unwrap().into();
        assert_eq!(res, decoded);
    }
}