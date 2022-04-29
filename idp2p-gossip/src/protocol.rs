use idp2p_core::did::identity::Identity;
use async_trait::async_trait;
use idp2p_common::serde_json;
use libp2p::{
    core::{
        upgrade::{read_length_prefixed, write_length_prefixed},
        ProtocolName,
    },
    futures::{AsyncRead, AsyncWrite, AsyncWriteExt},
    request_response::RequestResponseCodec,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum IdRequestPayload {
    // Register a peer(requires proof)
    Register {
        identity: Identity,
        subscriptions: Vec<String>,
        proof: String,
    },
    NodeMessage {
        id: String,
        message: IdRequestNodeMessage,
    },
    WalletMessage(String),
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum IdRequestNodeMessage {
    // Subscribe to identity
    Subscribe(String),
    // Publish message via gossipsub
    Publish { id: String, jwm: String },
    // Get identity information
    Get(String),
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum IdResponsePayload {
    Ok(IdResponsePayloadOk),
    Error(String),
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum  IdResponsePayloadOk {
    None,
    GetResult(Identity)
}

#[derive(Debug, Clone)]
pub struct IdProtocol();
#[derive(Clone)]
pub struct IdCodec();
#[derive(Debug, Clone, PartialEq)]
pub struct IdRequest(pub IdRequestPayload);
#[derive(Debug, Clone, PartialEq)]
pub struct IdResponse(pub IdResponsePayload);
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

        Ok(IdRequest(serde_json::from_slice(&vec)?))
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
        Ok(IdResponse(serde_json::from_slice(&vec)?))
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
        write_length_prefixed(io, serde_json::to_string(&data)?).await?;
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
        write_length_prefixed(io, serde_json::to_string(&data)?).await?;
        io.close().await?;

        Ok(())
    }
}