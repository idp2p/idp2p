use async_trait::async_trait;
use idp2p_proto::{
    pairsub_response::{BadRequest, GetOk, ResponseStatus},
    pubsub_message::PairsubMessage as ProtoPairsubMessage,
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
    access_token: String,
    message: PairsubRequestKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PairsubResponse {
    Ok,                         // Subscribe or Publish
    GetOk(Vec<PairsubMessage>), // Get
    UnAuthorized(String),
    BadRequest {
        id: String,
        code: String,
        message: String,
    },
    InternalError(String),
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
        let kind = match self.message.unwrap().pairsub_message.unwrap() {
            ProtoPairsubMessage::Resolve(_) => PairsubMessageKind::Resolve,
            ProtoPairsubMessage::Publish(payload) => PairsubMessageKind::Publish(payload),
            ProtoPairsubMessage::Evlepope(payload) => PairsubMessageKind::Publish(payload),
        };
        PairsubMessage { topic: self.topic, message: kind }
    }
}

impl Into<idp2p_proto::PairsubMessage> for PairsubMessage {
    fn into(self) -> idp2p_proto::PairsubMessage {
        todo!()
    }
}

impl Into<PairsubResponse> for idp2p_proto::PairsubResponse {
    fn into(self) -> PairsubResponse {
        match self.response_status.unwrap() {
            ResponseStatus::Ok(_) => todo!(),
            ResponseStatus::GetOk(_) => todo!(),
            ResponseStatus::Unauthorized(_) => todo!(),
            ResponseStatus::BadRequest(_) => todo!(),
            ResponseStatus::InternalError(_) => todo!(),
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
            PairsubResponse::UnAuthorized(error_id) => ResponseStatus::Unauthorized(error_id),
            PairsubResponse::BadRequest { id, code, message } => {
                ResponseStatus::BadRequest(BadRequest { id, code, message })
            }
            PairsubResponse::InternalError(error_id) => ResponseStatus::InternalError(error_id),
        };
        idp2p_proto::PairsubResponse {
            response_status: Some(status),
        }
    }
}

impl Into<PairsubRequest> for idp2p_proto::PairsubRequest {
    fn into(self) -> PairsubRequest {
        todo!()
    }
}

impl Into<idp2p_proto::PairsubRequest> for PairsubRequest {
    fn into(self) -> idp2p_proto::PairsubRequest {
        match self.message {
            PairsubRequestKind::Get => todo!(),
            PairsubRequestKind::Subscribe(_) => todo!(),
            PairsubRequestKind::Publish(_) => todo!(),
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
