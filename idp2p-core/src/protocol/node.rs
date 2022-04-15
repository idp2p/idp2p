// Simple Ping-Pong Protocol

use async_trait::async_trait;
use libp2p::{
    core::{
        upgrade::{read_length_prefixed, write_length_prefixed},
        ProtocolName,
    },
    futures::{AsyncRead, AsyncWrite, AsyncWriteExt},
    request_response::RequestResponseCodec,
};

#[derive(Debug, Clone)]
pub struct IdNodeProtocol();
#[derive(Clone)]
pub struct IdNodeCodec();
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdNodeRequest(pub String);
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdNodeResponse(pub String);

impl ProtocolName for IdNodeProtocol {
    fn protocol_name(&self) -> &[u8] {
        "/idp2p/node/1".as_bytes()
    }
}

#[async_trait]
impl RequestResponseCodec for IdNodeCodec {
    type Protocol = IdNodeProtocol;
    type Request = IdNodeRequest;
    type Response = IdNodeResponse;

    async fn read_request<T>(
        &mut self,
        _: &IdNodeProtocol,
        io: &mut T,
    ) -> std::io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let vec = read_length_prefixed(io, 1_000_000).await?;

        if vec.is_empty() {
            return Err(std::io::ErrorKind::UnexpectedEof.into());
        }

        Ok(IdNodeRequest(String::from_utf8(vec).unwrap()))
    }

    async fn read_response<T>(
        &mut self,
        _: &IdNodeProtocol,
        io: &mut T,
    ) -> std::io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let vec = read_length_prefixed(io, 1_000_000).await?;

        if vec.is_empty() {
            return Err(std::io::ErrorKind::UnexpectedEof.into());
        }
        Ok(IdNodeResponse(String::from_utf8(vec).unwrap()))
    }

    async fn write_request<T>(
        &mut self,
        _: &IdNodeProtocol,
        io: &mut T,
        IdNodeRequest(data): IdNodeRequest,
    ) -> std::io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_length_prefixed(io, data).await?;
        io.close().await?;

        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _: &IdNodeProtocol,
        io: &mut T,
        IdNodeResponse(data): IdNodeResponse,
    ) -> std::io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_length_prefixed(io, data).await?;
        io.close().await?;

        Ok(())
    }
}

pub async fn handle_request(req: &str) -> (){
    ()
}