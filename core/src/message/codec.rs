use async_trait::async_trait;
use libp2p::{
    core::{
        upgrade::{read_length_prefixed, write_length_prefixed},
        ProtocolName,
    },
    futures::{AsyncRead, AsyncWrite, AsyncWriteExt},
    request_response::RequestResponseCodec,
};
use prost::Message;

use super::IdMessage;

#[derive(Debug, Clone)]
pub struct IdProtocol();
#[derive(Clone)]
pub struct IdCodec();
#[derive(Debug, Clone, PartialEq)]
pub struct IdRequest(IdMessage);
#[derive(Debug, Clone, PartialEq)]
pub struct IdResponse(Vec<u8>);
impl ProtocolName for IdProtocol {
    fn protocol_name(&self) -> &[u8] {
        "/idp2p/1".as_bytes()
    }
}
#[async_trait]
impl RequestResponseCodec for IdCodec {
    type Protocol = IdProtocol;
    type Request = IdRequest;
    type Response = IdResponse;

    async fn read_request<T>(
        &mut self,
        _: &IdProtocol,
        io: &mut T,
    ) -> std::io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let vec = read_length_prefixed(io, 1_000_000).await?;

        if vec.is_empty() {
            return Err(std::io::ErrorKind::UnexpectedEof.into());
        }
        let msg_result = IdMessage::from_bytes(&vec);
        match msg_result{
           Ok(msg) =>  Ok(IdRequest(msg)),
           Err(_) => Err(std::io::ErrorKind::InvalidData.into())
        }
    }

    async fn read_response<T>(
        &mut self,
        _: &IdProtocol,
        io: &mut T,
    ) -> std::io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let vec = read_length_prefixed(io, 1_000_000).await?;

        if vec.is_empty() {
            return Err(std::io::ErrorKind::UnexpectedEof.into());
        }
        Ok(IdResponse(vec))
    }

    async fn write_request<T>(
        &mut self,
        _: &IdProtocol,
        io: &mut T,
        IdRequest(data): IdRequest,
    ) -> std::io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let msg: crate::idp2p_proto::IdMessage = data.into();
        write_length_prefixed(io, msg.encode_to_vec()).await?;
        io.close().await?;

        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _: &IdProtocol,
        io: &mut T,
        IdResponse(data): IdResponse,
    ) -> std::io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_length_prefixed(io, data).await?;
        io.close().await?;

        Ok(())
    }
}
