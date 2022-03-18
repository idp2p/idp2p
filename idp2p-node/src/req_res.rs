use libp2p::{
    core::upgrade::{read_length_prefixed, write_length_prefixed, ProtocolName},
    request_response::{
        RequestResponseCodec
    }
};
use libp2p::futures::io::{AsyncRead, AsyncWrite};
use libp2p::futures::AsyncWriteExt;
use async_trait::async_trait;
use std::io;

#[derive(Debug, Clone)]
pub struct IdExchangeProtocol();
#[derive(Clone)]
pub struct IdExchangeCodec();
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdRequest(String);
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdResponse(String);

impl ProtocolName for IdExchangeProtocol {
    fn protocol_name(&self) -> &[u8] {
        "/idp2p/1".as_bytes()
    }
}

#[async_trait]
impl RequestResponseCodec for IdExchangeCodec {
    type Protocol = IdExchangeProtocol;
    type Request = IdRequest;
    type Response = IdResponse;

    async fn read_request<T>(
        &mut self,
        _: &IdExchangeProtocol,
        io: &mut T,
    ) -> tokio::io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let vec = read_length_prefixed(io, 1_000_000).await?;
        if vec.is_empty() {
            return Err(tokio::io::ErrorKind::UnexpectedEof.into());
        }
        Ok(IdRequest(String::from_utf8(vec).unwrap()))
    }

    async fn read_response<T>(
        &mut self,
        _: &IdExchangeProtocol,
        io: &mut T,
    ) -> tokio::io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let vec = read_length_prefixed(io, 1_000_000).await?;

        if vec.is_empty() {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }

        Ok(IdResponse(String::from_utf8(vec).unwrap()))
    }

    async fn write_request<T>(
        &mut self,
        _: &IdExchangeProtocol,
        io: &mut T,
        IdRequest(data): IdRequest,
    ) -> tokio::io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_length_prefixed(io, data).await?;
        io.close().await?;

        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _: &IdExchangeProtocol,
        io: &mut T,
        IdResponse(data): IdResponse,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_length_prefixed(io, data).await?;
        io.close().await?;

        Ok(())
    }
}
