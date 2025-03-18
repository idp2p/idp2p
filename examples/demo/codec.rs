use futures::prelude::*;
use libp2p::swarm::StreamProtocol;
use std::io;

/// A codec for sending raw byte arrays over the request-response protocol.
/// This follows the structure of the CBOR codec but does no serialization.
pub struct RawCodec {
    /// Maximum request size in bytes.
    request_size_maximum: u64,
    /// Maximum response size in bytes.
    response_size_maximum: u64,
}

impl Default for RawCodec {
    fn default() -> Self {
        Self {
            request_size_maximum: 1024 * 1024,     // 1 MiB
            response_size_maximum: 10 * 1024 * 1024, // 10 MiB
        }
    }
}

impl RawCodec {
    /// Set the maximum allowed request size.
    pub fn set_request_size_maximum(mut self, request_size_maximum: u64) -> Self {
        self.request_size_maximum = request_size_maximum;
        self
    }

    /// Set the maximum allowed response size.
    pub fn set_response_size_maximum(mut self, response_size_maximum: u64) -> Self {
        self.response_size_maximum = response_size_maximum;
        self
    }
}

/// Implement the codec trait following the same structure as in the CBOR example.
/// Here we assume that the trait is available as `crate::Codec` (as in the CBOR codec).
#[async_trait::async_trait]
impl libp2p::request_response::Codec for RawCodec {
    type Protocol = StreamProtocol;
    type Request = Vec<u8>;
    type Response = Vec<u8>;

    async fn read_request<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        // Limit the incoming data to the maximum request size.
        io.take(self.request_size_maximum).read_to_end(&mut buf).await?;
        Ok(buf)
    }

    async fn read_response<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        // Limit the incoming data to the maximum response size.
        io.take(self.response_size_maximum).read_to_end(&mut buf).await?;
        Ok(buf)
    }

    async fn write_request<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
        req: Self::Request,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        io.write_all(&req).await?;
        io.close().await
    }

    async fn write_response<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
        resp: Self::Response,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        io.write_all(&resp).await?;
        io.close().await
    }
}
