use async_trait::async_trait;
use libp2p::{
    core::upgrade::{read_length_prefixed, write_length_prefixed},
    futures::{AsyncRead, AsyncWrite, AsyncWriteExt},
    request_response::{ProtocolName, RequestResponseCodec},
};
use pairsub_proto::{
    pairsub_request::PublishMessage,
    pairsub_request::{PairsubRequestKind as ProtoPairsubRequestKind, SubscribeMessage},
    pairsub_response::{GetResult, PairsubResponseError, ResponseStatus},
};
use prost::Message;
pub mod pairsub_proto {
    include!(concat!(env!("OUT_DIR"), "/pairsub.pb.rs"));
}

#[derive(Debug, Clone, PartialEq)]
pub struct PairsubMessage {
    topic: String,
    message: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PairsubRequestKind {
    Get,
    Notify,
    Subscribe(Vec<String>),
    UnSubscribe(Vec<String>),
    Publish(Vec<PairsubMessage>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PairsubRequest {
    pub pair_id: String,
    pub message: PairsubRequestKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PairsubResponse {
    Ok,                             // Subscribe or Publish
    GetResult(Vec<PairsubMessage>), // Get messages
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
        "/libp2p/pairsub/1".as_bytes()
    }
}

impl Into<PairsubMessage> for pairsub_proto::PairsubMessage {
    fn into(self) -> PairsubMessage {
        PairsubMessage {
            topic: self.topic,
            message: self.message,
        }
    }
}

impl Into<pairsub_proto::PairsubMessage> for PairsubMessage {
    fn into(self) -> pairsub_proto::PairsubMessage {
        pairsub_proto::PairsubMessage {
            topic: self.topic,
            message: self.message,
        }
    }
}

impl Into<PairsubRequest> for pairsub_proto::PairsubRequest {
    fn into(self) -> PairsubRequest {
        let kind = match self.pairsub_request_kind.unwrap() {
            ProtoPairsubRequestKind::Get(_) => PairsubRequestKind::Get,
            ProtoPairsubRequestKind::Subscribe(msg) => PairsubRequestKind::Subscribe(msg.topics),
            ProtoPairsubRequestKind::Publish(msg) => PairsubRequestKind::Publish(
                msg.messages
                    .into_iter()
                    .map(|x| x.into())
                    .collect::<Vec<PairsubMessage>>(),
            ),
            ProtoPairsubRequestKind::Notify(_) => todo!(),
            ProtoPairsubRequestKind::Unsubscribe(_) => todo!(),
        };
        PairsubRequest {
            pair_id: self.pair_id,
            message: kind,
        }
    }
}

impl Into<pairsub_proto::PairsubRequest> for PairsubRequest {
    fn into(self) -> pairsub_proto::PairsubRequest {
        let kind = match self.message {
            PairsubRequestKind::Get => ProtoPairsubRequestKind::Get(true),
            PairsubRequestKind::Subscribe(topics) => {
                ProtoPairsubRequestKind::Subscribe(SubscribeMessage { topics: topics })
            }
            PairsubRequestKind::Publish(msg) => ProtoPairsubRequestKind::Publish(PublishMessage {
                messages: msg
                    .into_iter()
                    .map(|x| x.into())
                    .collect::<Vec<pairsub_proto::PairsubMessage>>(),
            }),
            PairsubRequestKind::Notify => todo!(),
            PairsubRequestKind::UnSubscribe(_) => todo!(),
        };
        pairsub_proto::PairsubRequest {
            pair_id: self.pair_id,
            pairsub_request_kind: Some(kind),
        }
    }
}

impl Into<PairsubResponse> for pairsub_proto::PairsubResponse {
    fn into(self) -> PairsubResponse {
        match self.response_status.unwrap() {
            ResponseStatus::Ok(_) => PairsubResponse::Ok,
            ResponseStatus::GetResult(res) => {
                let mut messages: Vec<PairsubMessage> = vec![];
                for msg in res.messages {
                    messages.push(msg.into());
                }
                PairsubResponse::GetResult(messages)
            }
            ResponseStatus::Error(err) => PairsubResponse::Error {
                id: err.id,
                code: err.code,
                message: err.message,
            },
        }
    }
}

impl Into<pairsub_proto::PairsubResponse> for PairsubResponse {
    fn into(self) -> pairsub_proto::PairsubResponse {
        let status = match self {
            PairsubResponse::Ok => ResponseStatus::Ok(true),
            PairsubResponse::GetResult(messages) => {
                let mut p_messages: Vec<pairsub_proto::PairsubMessage> = vec![];
                for msg in messages {
                    p_messages.push(msg.into());
                }
                ResponseStatus::GetResult(GetResult {
                    messages: p_messages,
                })
            }
            PairsubResponse::Error { id, code, message } => {
                ResponseStatus::Error(PairsubResponseError { id, code, message })
            }
        };
        pairsub_proto::PairsubResponse {
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
        let req_proto = pairsub_proto::PairsubRequest::decode(&*vec)?;
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
        let res_proto = pairsub_proto::PairsubResponse::decode(&*vec)?;
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
        let req_proto: pairsub_proto::PairsubRequest = req.into();
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
        let res_proto: pairsub_proto::PairsubResponse = res.into();
        write_length_prefixed(io, res_proto.encode_to_vec()).await?;
        io.close().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pairsub_message_map_test() {
        let msg = PairsubMessage {
            topic: "Some".to_string(),
            message: vec![0u8; 32],
        };
        let proto_msg: pairsub_proto::PairsubMessage = msg.clone().into();
        let encoded = proto_msg.encode_to_vec();
        let decoded: PairsubMessage = pairsub_proto::PairsubMessage::decode(&*encoded)
            .unwrap()
            .into();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn pairsub_request_map_test() {
        let msg = PairsubMessage {
            topic: "Some".to_string(),
            message: vec![0u8; 32],
        };
        let req = PairsubRequest {
            pair_id: "Some".to_string(),
            message: PairsubRequestKind::Publish(vec![msg]),
        };
        let proto_req: pairsub_proto::PairsubRequest = req.clone().into();
        let encoded = proto_req.encode_to_vec();
        let decoded: PairsubRequest = pairsub_proto::PairsubRequest::decode(&*encoded)
            .unwrap()
            .into();
        assert_eq!(req, decoded);
    }

    #[test]
    fn pairsub_response_map_test() {
        let msg = PairsubMessage {
            topic: "Some".to_string(),
            message: vec![0u8; 32],
        };
        let res = PairsubResponse::GetResult(vec![msg]);
        let proto_res: pairsub_proto::PairsubResponse = res.clone().into();
        let encoded = proto_res.encode_to_vec();
        let decoded: PairsubResponse = pairsub_proto::PairsubResponse::decode(&*encoded)
            .unwrap()
            .into();
        assert_eq!(res, decoded);
    }
}
