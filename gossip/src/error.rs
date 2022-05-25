use core::{prost::DecodeError};

use libp2p::{gossipsub::error::PublishError, identity::error::DecodingError, TransportError, swarm::DialError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GossipError {
    #[error(transparent)]
    DecodeError(#[from] DecodeError),
    #[error(transparent)]
    PublishError(#[from] PublishError),
    #[error(transparent)]
    DecodingError(#[from] DecodingError),
    #[error(transparent)]
    MultiAddrError(#[from] libp2p::multiaddr::Error),
    #[error(transparent)]
    TransportError(#[from] TransportError<std::io::Error>),
    #[error(transparent)]
    DialError(#[from] DialError),
    #[error(transparent)]
    StdError(#[from] std::io::Error),
    #[error("Other")]
    Other,
}
