use async_trait::async_trait;
use idp2p_common::serde_json;
use idp2p_core::protocol::{IdNodeRequestPayload, IdResponsePayload};
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
#[derive(Debug, Clone, PartialEq)]
pub struct IdNodeRequest(pub IdNodeRequestPayload);
#[derive(Debug, Clone, PartialEq)]
pub struct IdNodeResponse(pub IdResponsePayload);
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

        Ok(IdNodeRequest(serde_json::from_slice(&vec)?))
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
        Ok(IdNodeResponse(serde_json::from_slice(&vec)?))
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
        write_length_prefixed(io, serde_json::to_string(&data)?).await?;
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
        write_length_prefixed(io, serde_json::to_string(&data)?).await?;
        io.close().await?;

        Ok(())
    }
}
